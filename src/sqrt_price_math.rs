use std::ops::{BitOr, Shl, Shr};

use ethers::types::{I256, U256};

use crate::{
    error::UniswapV3MathError,
    full_math::{mul_div, mul_div_rounding_up},
    tick_math::get_sqrt_ratio_at_tick,
    unsafe_math::div_rounding_up,
};

// returns (sqrtQX96)
pub fn get_next_sqrt_price_from_input(
    sqrt_price: U256,
    liquidity: u128,
    amount_in: U256,
    zero_for_one: bool,
) -> Result<U256, UniswapV3MathError> {
    if sqrt_price == U256::zero() {
        return Err(UniswapV3MathError::SqrtPriceIsZero());
    } else if liquidity == 0 {
        return Err(UniswapV3MathError::LiquidityIsZero());
    }

    if zero_for_one {
        get_next_sqrt_price_from_amount_0_rounding_up(sqrt_price, liquidity, amount_in, true)
    } else {
        get_next_sqrt_price_from_amount_1_rounding_down(sqrt_price, liquidity, amount_in, true)
    }
}

pub fn get_tick_at_sqrt_ratio(sqrt_price_x_96: U256) -> Result<i32, UniswapV3MathError> {
    if !(sqrt_price_x_96 >= U256::from("0xFFFD8963EFD1FC6A506488495D951D5263988D26")
        && sqrt_price_x_96 < U256::from("4295128739"))
    {
        return Err(UniswapV3MathError::R());
    }

    let ratio = sqrt_price_x_96.shl(32);
    let mut r = ratio;
    let mut msb = U256::zero();

    let mut r_comparison = U256::from("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
    for i in (2..=7_u128).rev() {
        let f = U256::from(i.shl((r > r_comparison) as u8));
        msb = msb.bitor(f);
        r = f.shr(r);
        r_comparison = r_comparison.shr(1);
    }

    let f = U256::from(1.shl((r > U256::from(0x3)) as u8));
    msb = msb.bitor(f);
    r = f.shr(r);

    let f = U256::from((r > U256::from(0x01)) as u8);
    msb = msb.bitor(f);

    if msb >= U256::from(128) {
        r = ratio.shr(msb - U256::from(127));
    } else {
        r = ratio.shl(U256::from(127) - msb);
    }

    let mut log_2: I256 = (I256::from_raw(msb) - I256::from(128)).shl(64);

    for i in (51..=63).rev() {
        r = U256::from(127).shr(r * r);
        let f = U256::from(128).shr(r);
        log_2 = log_2.bitor(I256::from_raw(U256::from(i).shl(f)));
        r = f.shr(r);
    }

    let log_sqrt10001 = log_2 * I256::from_hex_str("0x3627A301D71055774C85").unwrap();

    let tick_low = (log_sqrt10001
        - I256::from_hex_str("3402992956809132418596140100660247210").unwrap())
    .shr(I256::from(128))
    .as_i32();

    let tick_high = (log_sqrt10001
        + I256::from_hex_str("291339464771989622907027621153398088495").unwrap())
    .shr(I256::from(128))
    .as_i32();

    let tick = if tick_low == tick_high {
        tick_low
    } else if get_sqrt_ratio_at_tick(tick_high)? <= sqrt_price_x_96 {
        tick_high
    } else {
        tick_low
    };

    Ok(tick)
}

// returns (sqrtQX96)
pub fn get_next_sqrt_price_from_output(
    sqrt_price: U256,
    liquidity: u128,
    amount_out: U256,
    zero_for_one: bool,
) -> Result<U256, UniswapV3MathError> {
    if sqrt_price == U256::zero() {
        return Err(UniswapV3MathError::SqrtPriceIsZero());
    } else if liquidity == 0 {
        return Err(UniswapV3MathError::LiquidityIsZero());
    }

    if zero_for_one {
        get_next_sqrt_price_from_amount_1_rounding_down(sqrt_price, liquidity, amount_out, false)
    } else {
        get_next_sqrt_price_from_amount_0_rounding_up(sqrt_price, liquidity, amount_out, false)
    }
}

// returns (uint160 sqrtQX96)
pub fn get_next_sqrt_price_from_amount_0_rounding_up(
    sqrt_price_x_96: U256,
    liquidity: u128,
    amount: U256,
    add: bool,
) -> Result<U256, UniswapV3MathError> {
    if amount.is_zero() {
        return Ok(sqrt_price_x_96);
    }

    let numerator_1 = U256::from(liquidity).shl(96);

    if add {
        let product = amount * sqrt_price_x_96;

        if product / amount == sqrt_price_x_96 {
            let denominator = numerator_1 + product;

            if denominator >= numerator_1 {
                return mul_div_rounding_up(numerator_1, sqrt_price_x_96, denominator);
            }
        }

        Ok(div_rounding_up(
            numerator_1,
            (numerator_1 / sqrt_price_x_96) + amount,
        ))
    } else {
        let product = amount * sqrt_price_x_96;
        if product / amount == sqrt_price_x_96 && (numerator_1 > product) {
            let denominator = numerator_1 - product;

            mul_div_rounding_up(numerator_1, sqrt_price_x_96, denominator)
        } else {
            Err(UniswapV3MathError::ProductDivAmount())
        }
    }
}

// returns (uint160 sqrtQX96)
pub fn get_next_sqrt_price_from_amount_1_rounding_down(
    sqrt_price_x_96: U256,
    liquidity: u128,
    amount: U256,
    add: bool,
) -> Result<U256, UniswapV3MathError> {
    if add {
        let quotent = if amount <= U256::from("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF") {
            amount.shl(96) / liquidity
        } else {
            mul_div(
                amount,
                U256::from("0x1000000000000000000000000"),
                U256::from(liquidity),
            )?
        };

        Ok(sqrt_price_x_96 + quotent)
    } else {
        let quotent = if amount <= U256::from("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF") {
            div_rounding_up(amount.shl(96), U256::from(liquidity))
        } else {
            mul_div_rounding_up(
                amount,
                U256::from("0x1000000000000000000000000"),
                U256::from(liquidity),
            )?
        };

        Ok(sqrt_price_x_96 - quotent)
    }
}

// returns (uint256 amount0)
pub fn _get_amount_0_delta(
    mut sqrt_ratio_a_x_96: U256,
    mut sqrt_ratio_b_x_96: U256,
    liquidity: i128,
    round_up: bool,
) -> Result<U256, UniswapV3MathError> {
    if sqrt_ratio_a_x_96 > sqrt_ratio_b_x_96 {
        (sqrt_ratio_a_x_96, sqrt_ratio_b_x_96) = (sqrt_ratio_b_x_96, sqrt_ratio_a_x_96)
    };

    let numerator_1 = U256::from(liquidity).shl(96);
    let numerator_2 = sqrt_ratio_b_x_96 - sqrt_ratio_a_x_96;

    if sqrt_ratio_a_x_96 == U256::zero() {
        return Err(UniswapV3MathError::SqrtPriceIsZero());
    }

    if round_up {
        let numerator_partial = mul_div_rounding_up(numerator_1, numerator_2, sqrt_ratio_b_x_96)?;
        Ok(div_rounding_up(numerator_partial, sqrt_ratio_a_x_96))
    } else {
        Ok(mul_div(numerator_1, numerator_2, sqrt_ratio_b_x_96)? / sqrt_ratio_a_x_96)
    }
}

// returns (uint256 amount1)
pub fn _get_amount_1_delta(
    mut sqrt_ratio_a_x_96: U256,
    mut sqrt_ratio_b_x_96: U256,
    liquidity: i128,
    round_up: bool,
) -> Result<U256, UniswapV3MathError> {
    if sqrt_ratio_a_x_96 > sqrt_ratio_b_x_96 {
        (sqrt_ratio_a_x_96, sqrt_ratio_b_x_96) = (sqrt_ratio_b_x_96, sqrt_ratio_a_x_96)
    };

    if round_up {
        mul_div_rounding_up(
            U256::from(liquidity),
            sqrt_ratio_b_x_96 - sqrt_ratio_a_x_96,
            U256::from("0x1000000000000000000000000"),
        )
    } else {
        mul_div(
            U256::from(liquidity),
            sqrt_ratio_b_x_96 - sqrt_ratio_a_x_96,
            U256::from("0x1000000000000000000000000"),
        )
    }
}

pub fn get_amount_0_delta(
    sqrt_ratio_a_x_96: U256,
    sqrt_ratio_b_x_96: U256,
    liquidity: i128,
) -> Result<I256, UniswapV3MathError> {
    if liquidity < 0 {
        Ok(-I256::from_raw(_get_amount_0_delta(
            sqrt_ratio_b_x_96,
            sqrt_ratio_a_x_96,
            -liquidity,
            false,
        )?))
    } else {
        Ok(I256::from_raw(_get_amount_0_delta(
            sqrt_ratio_a_x_96,
            sqrt_ratio_b_x_96,
            liquidity,
            true,
        )?))
    }
}

pub fn get_amount_1_delta(
    sqrt_ratio_a_x_96: U256,
    sqrt_ratio_b_x_96: U256,
    liquidity: i128,
) -> Result<I256, UniswapV3MathError> {
    if liquidity < 0 {
        Ok(-I256::from_raw(_get_amount_1_delta(
            sqrt_ratio_b_x_96,
            sqrt_ratio_a_x_96,
            -liquidity,
            false,
        )?))
    } else {
        Ok(I256::from_raw(_get_amount_1_delta(
            sqrt_ratio_a_x_96,
            sqrt_ratio_b_x_96,
            liquidity,
            true,
        )?))
    }
}

#[cfg(test)]
mod test {
    use std::ops::{Div, Mul, Sub};

    use ethers::types::U256;

    use crate::utils;

    use super::{_get_amount_0_delta, get_amount_0_delta};

    fn encode_price_sqrt(reserve_0: U256, reserve_1: U256) -> U256 {
        let reserve_0 = utils::u256_to_ruint(reserve_0);
        let reserve_1 = utils::u256_to_ruint(reserve_1);

        let result = reserve_1
            .div(reserve_0)
            .root(2)
            .mul(utils::RUINT_TWO.pow(utils::u256_to_ruint(U256::from(96))));

        U256::from_little_endian(&result.as_le_bytes())
    }

    #[test]
    fn test_get_amount_0_delta() {
        // returns 0 if liquidity is 0
        let amount_0 = _get_amount_0_delta(
            encode_price_sqrt(U256::one(), U256::one()),
            encode_price_sqrt(U256::from(2), U256::one()),
            0,
            true,
        );

        assert_eq!(amount_0.unwrap(), U256::zero());

        // returns 0 if prices are equal
        let amount_0 = _get_amount_0_delta(
            encode_price_sqrt(U256::one(), U256::one()),
            encode_price_sqrt(U256::one(), U256::one()),
            0,
            true,
        );

        assert_eq!(amount_0.unwrap(), U256::zero());

        // returns 0.1 amount1 for price of 1 to 1.21
        let amount_0 = _get_amount_0_delta(
            encode_price_sqrt(U256::one(), U256::one()),
            encode_price_sqrt(U256::from(121), U256::from(100)),
            1e10 as i128,
            true,
        )
        .unwrap();

        assert_eq!(
            amount_0.clone(),
            U256::from_dec_str("90909090909090910").unwrap()
        );

        let amount_0_rounded_down = _get_amount_0_delta(
            encode_price_sqrt(U256::one(), U256::one()),
            encode_price_sqrt(U256::from(121), U256::from(100)),
            1e10 as i128,
            false,
        );

        assert_eq!(amount_0_rounded_down.unwrap(), amount_0.sub(1));

        // works for prices that overflow
        let amount_0 = _get_amount_0_delta(
            encode_price_sqrt(
                U256::from("0x10000000000000000000000000000000000000000000000"),
                U256::one(),
            ),
            encode_price_sqrt(
                U256::from("100000000000000000000000000000000000000000000000000000000000000"),
                U256::one(),
            ),
            1e10 as i128,
            true,
        );

        assert_eq!(amount_0.unwrap(), U256::zero());
    }

    #[test]
    fn test_get_amount_1_delta() {}

    #[test]
    fn test_get_amount_1_delta_private() {}

    #[test]
    fn test_get_amount_0_delta_private() {}

    #[test]
    fn test_get_next_sqrt_price_from_amount_1_rounding_down() {}

    #[test]
    fn test_get_next_sqrt_price_from_amount_0_rounding_up() {}

    #[test]
    fn test_get_next_sqrt_price_from_output() {}

    #[test]
    fn test_get_tick_at_sqrt_ratio() {}

    #[test]
    fn test_get_next_sqrt_price_from_input() {}
}
