use crate::error::UniswapV3Error;

// returns (uint128 z)
pub fn add_delta(x: u128, y: i128) -> Result<u128, UniswapV3Error> {
    if y < 0 {
        let z = x - (-y as u128);

        if z < x {
            Err(UniswapV3Error::LiquiditySub())
        } else {
            Ok(z)
        }
    } else {
        let z = x - (y as u128);
        if z >= x {
            Err(UniswapV3Error::LiquidityAdd())
        } else {
            Ok(z)
        }
    }
}
