use std::ops::Neg;

use ethers::types::{I256, U256};

use crate::{
    error::UniswapV3MathError,
    full_math::{mul_div, mul_div_rounding_up},
    sqrt_price_math::{
        _get_amount_0_delta, _get_amount_1_delta, get_next_sqrt_price_from_input,
        get_next_sqrt_price_from_output,
    },
};

// //returns (
//         uint160 sqrtRatioNextX96,
//         uint256 amountIn,
//         uint256 amountOut,
//         uint256 feeAmount
//     )
pub fn compute_swap_step(
    sqrt_ratio_current_x_96: U256,
    sqrt_ratio_target_x_96: U256,
    liquidity: u128,
    amount_remaining: I256,
    fee_pips: u32,
) -> Result<(U256, U256, U256, U256), UniswapV3MathError> {
    let zero_for_one = sqrt_ratio_current_x_96 > sqrt_ratio_target_x_96;
    let exact_in = amount_remaining >= I256::zero();

    let mut sqrt_ratio_next_x_96 = U256::zero();
    let mut amount_in = U256::zero();
    let mut amount_out = U256::zero();

    let negative_amount_in_remaining = (-amount_remaining).into_raw();

    if exact_in {
        let amount_remaining_less_fee = mul_div(
            amount_remaining.into_raw(),
            U256::from(1000000 - fee_pips), //1e6 - fee_pips
            U256::from(1000000),            //1e6
        )?;

        amount_in = if zero_for_one {
            _get_amount_0_delta(
                sqrt_ratio_target_x_96,
                sqrt_ratio_current_x_96,
                liquidity as i128,
                true,
            )?
        } else {
            _get_amount_1_delta(
                sqrt_ratio_current_x_96,
                sqrt_ratio_target_x_96,
                liquidity as i128,
                true,
            )?
        };

        if amount_remaining_less_fee >= amount_in {
            sqrt_ratio_next_x_96 = sqrt_ratio_target_x_96;
        } else {
            sqrt_ratio_next_x_96 = get_next_sqrt_price_from_input(
                sqrt_ratio_current_x_96,
                liquidity,
                amount_remaining_less_fee,
                zero_for_one,
            )?;
        }
    } else {
        amount_out = if zero_for_one {
            _get_amount_1_delta(
                sqrt_ratio_target_x_96,
                sqrt_ratio_current_x_96,
                liquidity as i128,
                false,
            )?
        } else {
            _get_amount_0_delta(
                sqrt_ratio_current_x_96,
                sqrt_ratio_target_x_96,
                liquidity as i128,
                false,
            )?
        };

        sqrt_ratio_next_x_96 = if negative_amount_in_remaining >= amount_out {
            sqrt_ratio_target_x_96
        } else {
            get_next_sqrt_price_from_output(
                sqrt_ratio_current_x_96,
                liquidity,
                negative_amount_in_remaining,
                zero_for_one,
            )?
        };
    }

    let max = sqrt_ratio_target_x_96 == sqrt_ratio_next_x_96;

    if zero_for_one {
        if !max || !exact_in {
            amount_in = _get_amount_0_delta(
                sqrt_ratio_next_x_96,
                sqrt_ratio_current_x_96,
                liquidity as i128,
                true,
            )?
        }

        if !max || exact_in {
            amount_out = _get_amount_1_delta(
                sqrt_ratio_next_x_96,
                sqrt_ratio_current_x_96,
                liquidity as i128,
                false,
            )?
        }
    } else {
        if !max || !exact_in {
            amount_in = _get_amount_1_delta(
                sqrt_ratio_current_x_96,
                sqrt_ratio_next_x_96,
                liquidity as i128,
                true,
            )?
        }

        if !max || exact_in {
            amount_out = _get_amount_0_delta(
                sqrt_ratio_current_x_96,
                sqrt_ratio_next_x_96,
                liquidity as i128,
                false,
            )?
        }
    }

    if !exact_in && amount_out > negative_amount_in_remaining {
        amount_out = negative_amount_in_remaining;
    }

    if exact_in && sqrt_ratio_next_x_96 != sqrt_ratio_target_x_96 {
        let amount_remaining_u256 = amount_remaining.into_raw();

        let fee_amount = amount_remaining_u256 - amount_in;
        Ok((sqrt_ratio_next_x_96, amount_in, amount_out, fee_amount))
    } else {
        let fee_amount = mul_div_rounding_up(
            amount_in,
            U256::from(fee_pips),
            U256::from(1000000 - fee_pips),
        )?;

        Ok((sqrt_ratio_next_x_96, amount_in, amount_out, fee_amount))
    }
}

mod test {
    use ethers::types::{I256, U256};

    use crate::swap_math::compute_swap_step;

    #[test]
    fn test_compute_swap_step() {
        //exact amount in that gets capped at price target in one for zero
        //Fails if price is zero
        let (sqrt_p, amount_in, amount_out, fee_amount) = compute_swap_step(
            U256::from_dec_str("79228162514264337593543950336").unwrap(),
            U256::from_dec_str("79623317895830914510639640423").unwrap(),
            2e18 as u128,
            I256::from_dec_str("1000000000000000000").unwrap(),
            600,
        )
        .unwrap();

        assert_eq!(
            sqrt_p,
            U256::from_dec_str("79623317895830914510639640423").unwrap()
        );
        assert_eq!(amount_out, U256::from_dec_str("9925619580021728").unwrap());
        assert_eq!(amount_in, U256::from_dec_str("9975124224178055").unwrap());
        assert_eq!(fee_amount, U256::from_dec_str("5988667735148").unwrap());

        let (sqrt_p, amount_in, amount_out, fee_amount) = compute_swap_step(
            U256::from_dec_str("79228162514264337593543950336").unwrap(),
            U256::from_dec_str("79623317895830914510639640424").unwrap(),
            2e18 as u128,
            I256::from_dec_str("-1000000000000000000").unwrap(),
            600,
        )
        .unwrap();

        assert_eq!(
            sqrt_p,
            U256::from_dec_str("79623317895830914510639640424").unwrap()
        );
        assert_eq!(amount_out, U256::from_dec_str("9925619580021728").unwrap());
        assert_eq!(amount_in, U256::from_dec_str("9975124224178055").unwrap());
        assert_eq!(fee_amount, U256::from_dec_str("5988667735148").unwrap());
        assert!(amount_out < (U256::from_dec_str("1000000000000000000").unwrap()));

        //exact amount in that is fully spent in one for zero
        let (_, amount_in, amount_out, fee_amount) = compute_swap_step(
            U256::from_dec_str("79228162514264337593543950336").unwrap(),
            U256::from_dec_str("250541448375047931186413801569").unwrap(),
            2e18 as u128,
            I256::from_dec_str("1000000000000000000").unwrap(),
            600,
        )
        .unwrap();

        assert_eq!(
            amount_out,
            U256::from_dec_str("666399946655997866").unwrap()
        );
        assert_eq!(amount_in, U256::from_dec_str("999400000000000000").unwrap());
        assert_eq!(fee_amount, U256::from_dec_str("600000000000000").unwrap());
        assert_eq!(
            amount_in + fee_amount,
            U256::from_dec_str("1000000000000000000").unwrap()
        );

        //exact amount out that is fully received in one for zero
        let (_, amount_in, amount_out, fee_amount) = compute_swap_step(
            U256::from_dec_str("79228162514264337593543950336").unwrap(),
            U256::from_dec_str("792281625142643375935439503360").unwrap(),
            2e18 as u128,
            I256::from_dec_str("-1000000000000000000").unwrap(),
            600,
        )
        .unwrap();

        assert_eq!(
            amount_out,
            U256::from_dec_str("1000000000000000000").unwrap()
        );
        assert_eq!(
            amount_in,
            U256::from_dec_str("2000000000000000000").unwrap()
        );
        assert_eq!(fee_amount, U256::from_dec_str("1200720432259356").unwrap());

        //amount out is capped at the desired amount out
        let (sqrt_p, amount_in, amount_out, fee_amount) = compute_swap_step(
            U256::from_dec_str("417332158212080721273783715441582").unwrap(),
            U256::from_dec_str("1452870262520218020823638996").unwrap(),
            159344665391607089467575320103 as u128,
            I256::from_dec_str("-1").unwrap(),
            1,
        )
        .unwrap();

        assert_eq!(amount_out, U256::from_dec_str("1").unwrap());
        assert_eq!(amount_in, U256::from_dec_str("1").unwrap());
        assert_eq!(fee_amount, U256::from_dec_str("1").unwrap());
        assert_eq!(
            sqrt_p,
            U256::from_dec_str("417332158212080721273783715441581").unwrap()
        );

        //target price of 1 uses partial input amount
        let (sqrt_p, amount_in, amount_out, fee_amount) = compute_swap_step(
            U256::from_dec_str("2").unwrap(),
            U256::from_dec_str("1").unwrap(),
            1 as u128,
            I256::from_dec_str("3915081100057732413702495386755767").unwrap(),
            1,
        )
        .unwrap();

        assert_eq!(amount_out, U256::from_dec_str("0").unwrap());
        assert_eq!(
            amount_in,
            U256::from_dec_str("39614081257132168796771975168").unwrap()
        );
        assert_eq!(
            fee_amount,
            U256::from_dec_str("39614120871253040049813").unwrap()
        );
        assert_eq!(sqrt_p, U256::from_dec_str("1").unwrap());

        //entire input amount taken as fee
        let (sqrt_p, amount_in, amount_out, fee_amount) = compute_swap_step(
            U256::from_dec_str("2413").unwrap(),
            U256::from_dec_str("79887613182836312").unwrap(),
            1985041575832132834610021537970 as u128,
            I256::from_dec_str("10").unwrap(),
            1872,
        )
        .unwrap();

        assert_eq!(amount_out, U256::from_dec_str("0").unwrap());
        assert_eq!(amount_in, U256::from_dec_str("0").unwrap());
        assert_eq!(fee_amount, U256::from_dec_str("10").unwrap());
        assert_eq!(sqrt_p, U256::from_dec_str("2413").unwrap());

        //handles intermediate insufficient liquidity in zero for one exact output case
        let (sqrt_p, amount_in, amount_out, fee_amount) = compute_swap_step(
            U256::from_dec_str("20282409603651670423947251286016").unwrap(),
            U256::from_dec_str("22310650564016837466341976414617").unwrap(),
            1024 as u128,
            I256::from_dec_str("-4").unwrap(),
            3000,
        )
        .unwrap();

        assert_eq!(amount_out, U256::from_dec_str("0").unwrap()); //Getting a 1 wei rounding error here
        assert_eq!(amount_in, U256::from_dec_str("26215").unwrap());
        assert_eq!(fee_amount, U256::from_dec_str("79").unwrap());
        assert_eq!(
            sqrt_p,
            U256::from_dec_str("22310650564016837466341976414617").unwrap()
        );

        //handles intermediate insufficient liquidity in one for zero exact output case
        let (sqrt_p, amount_in, amount_out, fee_amount) = compute_swap_step(
            U256::from_dec_str("20282409603651670423947251286016").unwrap(),
            U256::from_dec_str("18254168643286503381552526157414").unwrap(),
            1024 as u128,
            I256::from_dec_str("-263000").unwrap(),
            3000,
        )
        .unwrap();

        assert_eq!(amount_out, U256::from_dec_str("26214").unwrap());
        assert_eq!(amount_in, U256::one());
        assert_eq!(fee_amount, U256::one());
        assert_eq!(
            sqrt_p,
            U256::from_dec_str("18254168643286503381552526157414").unwrap()
        );
    }
}
