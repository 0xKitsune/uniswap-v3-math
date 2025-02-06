pub use liquidity_amounts::*;

use crate::error::UniswapV3MathError;

/// Add a signed liquidity delta to liquidity and revert if it overflows or underflows. Returns (uint128 z)
/// - x: The liquidity before change
/// - y: The delta by which liquidity should be changed
pub fn add_delta(x: u128, y: i128) -> Result<u128, UniswapV3MathError> {
    if y < 0 {
        let z = x.overflowing_sub(-y as u128);

        if z.1 {
            Err(UniswapV3MathError::LiquiditySub)
        } else {
            Ok(z.0)
        }
    } else {
        let z = x.overflowing_add(y as u128);
        if z.0 < x {
            Err(UniswapV3MathError::LiquidityAdd)
        } else {
            Ok(z.0)
        }
    }
}

/// Provides functions for computing liquidity amounts from token amounts and prices
mod liquidity_amounts {
    use crate::{
        error::UniswapV3MathError,
        full_math::mul_div,
        sqrt_price_math::{FIXED_POINT_96_RESOLUTION, Q96}
    };
    use alloy::primitives::U256;

    /// Computes the amount of liquidity received for a given amount of token0 and price range. Returns (uint128 liquidity).
    /// Calculates amount0 * (sqrt(upper) * sqrt(lower)) / (sqrt(upper) - sqrt(lower))
    /// 
    /// - sqrt_ratio_a_x96: A sqrt price representing the first tick boundary
    /// - sqrt_ratio_b_x96: A sqrt price representing the second tick boundary
    /// - amount0: The amount0 being sent in
    pub fn get_liquidity_for_amount0(
        sqrt_ratio_a_x96: U256,
        sqrt_ratio_b_x96: U256,
        amount0: U256,
    ) -> Result<u128, UniswapV3MathError> {
        let diff = if sqrt_ratio_a_x96 > sqrt_ratio_b_x96 {
            sqrt_ratio_a_x96 - sqrt_ratio_b_x96
        } else {
            sqrt_ratio_b_x96 - sqrt_ratio_a_x96
        };
        let intermediate = mul_div(sqrt_ratio_a_x96, sqrt_ratio_b_x96, Q96)?;
        let liquidity = mul_div(amount0, intermediate, diff)?;

        Ok(liquidity.to::<u128>())
    }

    /// Computes the amount of liquidity received for a given amount of token1 and price range. Returns (uint128 liquidity).
    /// Calculates amount1 / (sqrt(upper) - sqrt(lower)).
    /// 
    /// - sqrt_ratio_a_x96: A sqrt price representing the first tick boundary
    /// - sqrt_ratio_b_x96: A sqrt price representing the second tick boundary
    /// - amount1: The amount1 being sent in
    pub fn get_liquidity_for_amount1(
        sqrt_ratio_a_x96: U256,
        sqrt_ratio_b_x96: U256,
        amount1: U256,
    ) -> Result<u128, UniswapV3MathError> {
        let diff = if sqrt_ratio_a_x96 > sqrt_ratio_b_x96 {
            sqrt_ratio_a_x96 - sqrt_ratio_b_x96
        } else {
            sqrt_ratio_b_x96 - sqrt_ratio_a_x96
        };
        let liquidity = mul_div(amount1, Q96, diff)?;

        Ok(liquidity.to::<u128>())
    }

    /// Computes the maximum amount of liquidity received for a given amount of token0, token1, the current
    /// pool prices and the prices at the tick boundaries. Returns (uint128 liquidity).
    /// 
    /// - sqrt_ratio_x96: A sqrt price representing the current pool prices
    /// - sqrt_ratio_a_x96: A sqrt price representing the first tick boundary
    /// - sqrt_ratio_b_x96: A sqrt price representing the second tick boundary
    /// - amount0: The amount of token0 being sent in
    /// - amount1: The amount of token1 being sent in
    pub fn get_liquidity_for_amounts(
        sqrt_ratio_x96: U256,
        sqrt_ratio_a_x96: U256,
        sqrt_ratio_b_x96: U256,
        amount0: U256,
        amount1: U256,
    ) -> Result<u128, UniswapV3MathError> {
        let (sqrt_ratio_a_x96, sqrt_ratio_b_x96) = if sqrt_ratio_a_x96 > sqrt_ratio_b_x96 {
            (sqrt_ratio_b_x96, sqrt_ratio_a_x96)
        } else {
            (sqrt_ratio_a_x96, sqrt_ratio_b_x96)
        };

        if sqrt_ratio_x96 <= sqrt_ratio_a_x96 {
            get_liquidity_for_amount0(sqrt_ratio_a_x96, sqrt_ratio_b_x96, amount0)
        } else if sqrt_ratio_x96 < sqrt_ratio_b_x96 {
            let liq0 = get_liquidity_for_amount0(sqrt_ratio_x96, sqrt_ratio_b_x96, amount0)?;
            let liq1 = get_liquidity_for_amount1(sqrt_ratio_a_x96, sqrt_ratio_x96, amount1)?;

            Ok(std::cmp::min(liq0, liq1))
        } else {
            get_liquidity_for_amount1(sqrt_ratio_a_x96, sqrt_ratio_b_x96, amount1)
        }
    }

    /// Computes the amount of token0 for a given amount of liquidity and a price range. Returns (uint256 amount0).
    /// 
    /// - sqrt_ratio_a_x96: A sqrt price representing the first tick boundary
    /// - sqrt_ratio_b_x96: A sqrt price representing the second tick boundary
    /// - liquidity: The liquidity being valued
    pub fn get_amount0_for_liquidity(
        sqrt_ratio_a_x96: U256,
        sqrt_ratio_b_x96: U256,
        liquidity: u128
    ) -> Result<U256, UniswapV3MathError> {
        if sqrt_ratio_a_x96.is_zero() || sqrt_ratio_b_x96.is_zero() { return Err(UniswapV3MathError::SqrtPriceIsZero) };

        let diff = if sqrt_ratio_a_x96 > sqrt_ratio_b_x96 {
            sqrt_ratio_a_x96 - sqrt_ratio_b_x96
        } else {
            sqrt_ratio_b_x96 - sqrt_ratio_a_x96
        };

        let numerator = mul_div(
            U256::from(liquidity) << FIXED_POINT_96_RESOLUTION,
            diff,
            sqrt_ratio_b_x96
        )?;

        Ok(numerator / sqrt_ratio_a_x96)
    }

    /// Computes the amount of token1 for a given amount of liquidity and a price range. Returns (uint256 amount1).
    /// 
    /// - sqrt_ratio_a_x96: A sqrt price representing the first tick boundary
    /// - sqrt_ratio_b_x96: A sqrt price representing the second tick boundary
    /// - liquidity: The liquidity being valued
    pub fn get_amount1_for_liquidity(
        sqrt_ratio_a_x96: U256,
        sqrt_ratio_b_x96: U256,
        liquidity: u128
    ) -> Result<U256, UniswapV3MathError> {
        let diff = if sqrt_ratio_a_x96 > sqrt_ratio_b_x96 {
            sqrt_ratio_a_x96 - sqrt_ratio_b_x96
        } else {
            sqrt_ratio_b_x96 - sqrt_ratio_a_x96
        };

        mul_div(U256::from(liquidity), diff, Q96)
    }

    /// Computes the token0 and token1 value for a given amount of liquidity, the current
    /// pool prices and the prices at the tick boundaries. Returns (uint256 amount0, uint256 amount1).
    /// 
    /// - sqrt_ratio_x96: A sqrt price representing the current pool prices
    /// - sqrt_ratio_a_x96: A sqrt price representing the first tick boundary
    /// - sqrt_ratio_b_x96: A sqrt price representing the second tick boundary
    /// - liquidity: The liquidity being valued
    pub fn get_amounts_for_liquidity(
        sqrt_ratio_x96: U256,
        sqrt_ratio_a_x96: U256,
        sqrt_ratio_b_x96: U256,
        liquidity: u128
    ) -> Result<(U256, U256), UniswapV3MathError> {
        let (sqrt_ratio_a_x96, sqrt_ratio_b_x96) = if sqrt_ratio_a_x96 > sqrt_ratio_b_x96 {
            (sqrt_ratio_b_x96, sqrt_ratio_a_x96)
        } else {
            (sqrt_ratio_a_x96, sqrt_ratio_b_x96)
        };

        let (amount0, amount1) = if sqrt_ratio_x96 <= sqrt_ratio_a_x96 {
            (
                get_amount0_for_liquidity(sqrt_ratio_a_x96, sqrt_ratio_b_x96, liquidity)?,
                U256::ZERO
            )
        } else if sqrt_ratio_x96 < sqrt_ratio_b_x96 {
            (
                get_amount0_for_liquidity(sqrt_ratio_x96, sqrt_ratio_b_x96, liquidity)?,
                get_amount1_for_liquidity(sqrt_ratio_a_x96, sqrt_ratio_x96, liquidity)?
            )
        } else {
            (
                U256::ZERO,
                get_amount1_for_liquidity(sqrt_ratio_a_x96, sqrt_ratio_b_x96, liquidity)?
            )
        };
        
        Ok((amount0, amount1))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tick_math::get_sqrt_ratio_at_tick;
    use alloy::primitives::U256;

    #[test]
    fn test_add_delta() {
        // 1 + 0
        let result = add_delta(1, 0);
        assert_eq!(result.unwrap(), 1);

        // 1 + -1
        let result = add_delta(1, -1);
        assert_eq!(result.unwrap(), 0);

        // 1 + 1
        let result = add_delta(1, 1);
        assert_eq!(result.unwrap(), 2);

        // 2**128-15 + 15 overflows
        let result = add_delta(340282366920938463463374607431768211441, 15);
        assert_eq!(result.err().unwrap().to_string(), "Liquidity Add");

        // 0 + -1 underflows
        let result = add_delta(0, -1);
        assert_eq!(result.err().unwrap().to_string(), "Liquidity Sub");

        // 3 + -4 underflows
        let result = add_delta(3, -4);
        assert_eq!(result.err().unwrap().to_string(), "Liquidity Sub");
    }

    
    #[test]
    fn test_get_amount_from_range() {
        // define a liquidity range
        let liquidity = 22402462192838616433_u128;
        let range_low = get_sqrt_ratio_at_tick(195540).unwrap();
        let range_high = get_sqrt_ratio_at_tick(195600).unwrap();

        // scenarios based on tick:
        let lower = get_sqrt_ratio_at_tick(190000).unwrap();
        let in_between = get_sqrt_ratio_at_tick(195574).unwrap();
        let higher = get_sqrt_ratio_at_tick(200000).unwrap(); 

        // Expected amounts computed with: https://github.com/Uniswap/v3-periphery/blob/main/contracts/libraries/LiquidityAmounts.sol

        // scenario: tick < range.low
        let (amount0, amount1) = get_amounts_for_liquidity(lower, range_low, range_high, liquidity).unwrap();
        assert_eq!(amount0, U256::from(3809422905322_u128));
        assert_eq!(amount1, U256::ZERO);

        // scenario: tick = range.low
        let (amount0, amount1) = get_amounts_for_liquidity(range_low, range_low, range_high, liquidity).unwrap();
        assert_eq!(amount0, U256::from(3809422905322_u128));
        assert_eq!(amount1, U256::ZERO);

        // scenario: range.low < tick < range.high
        let (amount0, amount1) = get_amounts_for_liquidity(in_between, range_low, range_high, liquidity).unwrap();
        assert_eq!(amount0, U256::from(1649346952146_u128));
        assert_eq!(amount1, U256::from(671393300975951287166_u128));

        // scenario: range.high = tick
        let (amount0, amount1) = get_amounts_for_liquidity(range_high, range_low, range_high, liquidity).unwrap();
        assert_eq!(amount0, U256::ZERO);
        assert_eq!(amount1, U256::from(1185582348830684008921_u128));

        // scenario: range.high < tick
        let (amount0, amount1) = get_amounts_for_liquidity(higher, range_low, range_high, liquidity).unwrap();
        assert_eq!(amount0, U256::ZERO);
        assert_eq!(amount1, U256::from(1185582348830684008921_u128));
    }

    #[test]
    fn test_get_liquidity_from_amounts() {
        // define a liquidity range
        let amount0 = U256::from(5_000_000e6);   // 5M USDC
        let amount1 = U256::from(1_000e18);      // 1k WETH
        let range_low = get_sqrt_ratio_at_tick(195540).unwrap();
        let range_high = get_sqrt_ratio_at_tick(195600).unwrap();

        // scenarios based on tick:
        let lower = get_sqrt_ratio_at_tick(190000).unwrap();
        let in_between = get_sqrt_ratio_at_tick(195574).unwrap();
        let higher = get_sqrt_ratio_at_tick(200000).unwrap(); 

        // Expected amounts computed with: https://github.com/Uniswap/v3-periphery/blob/main/contracts/libraries/LiquidityAmounts.sol

        // scenario: tick < range.low
        let liquidity = get_liquidity_for_amounts(lower, range_low, range_high, amount0, amount1).unwrap();
        assert_eq!(liquidity, 29404010462500336572_u128);

        // scenario: tick = range.low
        let liquidity = get_liquidity_for_amounts(range_low, range_low, range_high, amount0, amount1).unwrap();
        assert_eq!(liquidity, 29404010462500336572_u128);

        // scenario: range.low < tick < range.high
        let liquidity = get_liquidity_for_amounts(in_between, range_low, range_high, amount0, amount1).unwrap();
        assert_eq!(liquidity, 33367122013689935176_u128);

        // scenario: range.high = tick
        let liquidity = get_liquidity_for_amounts(range_high, range_low, range_high, amount0, amount1).unwrap();
        assert_eq!(liquidity, 18895745381949818729_u128);

        // scenario: range.high < tick
        let liquidity = get_liquidity_for_amounts(higher, range_low, range_high, amount0, amount1).unwrap();
        assert_eq!(liquidity, 18895745381949818729_u128);
    }
}
