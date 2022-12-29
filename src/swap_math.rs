use std::{ops::Neg};

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

    let negative_amount_in_remaining = amount_remaining.neg().into_raw();

    if exact_in {
        let amount_remaining_less_fee = mul_div(
            amount_remaining.into_raw(),
            U256::from(1000000 - fee_pips), //1e6 - fee_pips
            U256::from(1000000),            //1e6
        )?;

        amount_in = if zero_for_one {
            _get_amount_1_delta(
                sqrt_ratio_target_x_96,
                sqrt_ratio_current_x_96,
                liquidity as i128,
                true,
            )?
        } else {
            _get_amount_0_delta(
                sqrt_ratio_current_x_96,
                sqrt_ratio_target_x_96,
                liquidity as i128,
                true,
            )?
        };

        if amount_remaining_less_fee < amount_in {
            sqrt_ratio_next_x_96 = get_next_sqrt_price_from_input(
                sqrt_ratio_current_x_96,
                liquidity,
                amount_remaining_less_fee,
                zero_for_one,
            )?
        }
    } else {
        amount_out = if zero_for_one {
            _get_amount_1_delta(
                sqrt_ratio_target_x_96,
                sqrt_ratio_current_x_96,
                liquidity as i128,
                true,
            )?
        } else {
            _get_amount_0_delta(
                sqrt_ratio_current_x_96,
                sqrt_ratio_target_x_96,
                liquidity as i128,
                true,
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
