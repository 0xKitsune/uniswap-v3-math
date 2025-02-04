use crate::{
    error::UniswapV3MathError,
    full_math::mul_div,
    sqrt_price_math::{FIXED_POINT_96_RESOLUTION, Q96}
};
use alloy::primitives::U256;

// Downcasts U256 to u128
fn to_u128(x: U256) -> Result<u128, UniswapV3MathError> {
    x.to_string().parse().map_err(|_| UniswapV3MathError::SafeCastToU128Overflow)
}

// Computes the amount of liquidity received for a given amount of token0 and price range. Returns (uint128 liquidity)
pub fn get_liquidity_for_amount0(
    mut sqrt_pa: U256,
    mut sqrt_pb: U256,
    amount0: U256,
) -> Result<u128, UniswapV3MathError> {
    if sqrt_pa > sqrt_pb { std::mem::swap(&mut sqrt_pa, &mut sqrt_pb) };
    let intermediate = mul_div(sqrt_pa, sqrt_pb, Q96)?;
    let liquidity = mul_div(amount0, intermediate, sqrt_pb - sqrt_pa)?;

    to_u128(liquidity)
}

// Computes the amount of liquidity received for a given amount of token1 and price range. Returns (uint128 liquidity)
pub fn get_liquidity_for_amount1(
    mut sqrt_pa: U256,
    mut sqrt_pb: U256,
    amount1: U256,
) -> Result<u128, UniswapV3MathError> {
    if sqrt_pa > sqrt_pb { std::mem::swap(&mut sqrt_pa, &mut sqrt_pb) };
    let liquidity = mul_div(amount1, Q96, sqrt_pb - sqrt_pa)?;

    to_u128(liquidity)
}

// Computes the maximum amount of liquidity received for a given amount of token0, token1, the current
// pool prices and the prices at the tick boundaries. Returns (uint128 liquidity)
pub fn get_liquidity_for_amounts(
    sqrt_p: U256,
    mut sqrt_pa: U256,
    mut sqrt_pb: U256,
    amount0: U256,
    amount1: U256,
) -> Result<u128, UniswapV3MathError> {
    if sqrt_pa > sqrt_pb { std::mem::swap(&mut sqrt_pa, &mut sqrt_pb) };

    if sqrt_p <= sqrt_pa {
        get_liquidity_for_amount0(sqrt_pa, sqrt_pb, amount0)
    } else if sqrt_p < sqrt_pb {
        let liq0 = get_liquidity_for_amount0(sqrt_p, sqrt_pb, amount0)?;
        let liq1 = get_liquidity_for_amount1(sqrt_pa, sqrt_p, amount1)?;

        Ok(std::cmp::min(liq0, liq1))
    } else {
        get_liquidity_for_amount1(sqrt_pa, sqrt_pb, amount1)
    }
}

// Computes the amount of token0 for a given amount of liquidity and a price range. Returns (uint256 amount0)
pub fn get_amount0_for_liquidity(
    mut sqrt_pa: U256,
    mut sqrt_pb: U256,
    liquidity: u128
) -> Result<U256, UniswapV3MathError> {
    if sqrt_pa > sqrt_pb { std::mem::swap(&mut sqrt_pa, &mut sqrt_pb) };
    if sqrt_pa.is_zero() { return Err(UniswapV3MathError::SqrtPriceIsZero) };

    let numerator = mul_div(
        U256::from(liquidity) << FIXED_POINT_96_RESOLUTION,
        sqrt_pb - sqrt_pa,
        sqrt_pb
    )?;

    Ok(numerator / sqrt_pa)
}

// Computes the amount of token1 for a given amount of liquidity and a price range. Returns (uint256 amount1)
pub fn get_amount1_for_liquidity(
    mut sqrt_pa: U256,
    mut sqrt_pb: U256,
    liquidity: u128
) -> Result<U256, UniswapV3MathError> {
    if sqrt_pa > sqrt_pb { std::mem::swap(&mut sqrt_pa, &mut sqrt_pb) };

    mul_div(U256::from(liquidity), sqrt_pb - sqrt_pa, Q96)
}

// Computes the token0 and token1 value for a given amount of liquidity, the current
// pool prices and the prices at the tick boundaries. Returns (uint256 amount0, uint156 amount1)
pub fn get_amounts_for_liquidity(
    sqrt_p: U256,
    mut sqrt_pa: U256,
    mut sqrt_pb: U256,
    liquidity: u128
) -> Result<(U256, U256), UniswapV3MathError> {
    if sqrt_pa > sqrt_pb { std::mem::swap(&mut sqrt_pa, &mut sqrt_pb) };

    let (amount0, amount1) = if sqrt_p <= sqrt_pa {
        (get_amount0_for_liquidity(sqrt_pa, sqrt_pb, liquidity)?, U256::ZERO)
    } else if sqrt_p < sqrt_pb {
        (
            get_amount0_for_liquidity(sqrt_p, sqrt_pb, liquidity)?,
            get_amount1_for_liquidity(sqrt_pa, sqrt_p, liquidity)?
        )
    } else {
        (U256::ZERO, get_amount1_for_liquidity(sqrt_pa, sqrt_pb, liquidity)?)
    };
    
    Ok((amount0, amount1))
}

// returns (uint128 z)
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

#[cfg(test)]
mod test {
    use crate::{liquidity_math::*, tick_math::get_sqrt_ratio_at_tick};

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
