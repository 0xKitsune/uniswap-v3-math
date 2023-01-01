use ethers::types::U256;

use crate::error::UniswapV3MathError;

pub fn most_significant_bit(x: U256) -> Result<u8, UniswapV3MathError> {
    if x.is_zero() {
        return Err(UniswapV3MathError::ZeroValue());
    }

    let be_bytes = &mut [0u8; 32];
    x.to_big_endian(be_bytes);
    Ok(be_bytes[0])
}

pub fn least_significant_bit(x: U256) -> Result<u8, UniswapV3MathError> {
    if x.is_zero() {
        return Err(UniswapV3MathError::ZeroValue());
    }

    let le_bytes = &mut [0u8; 32];
    x.to_little_endian(le_bytes);
    Ok(le_bytes[0])
}

#[cfg(test)]
mod test {

    use ethers::types::U256;

    use crate::error::UniswapV3MathError;

    use super::most_significant_bit;

    #[test]
    fn test_most_significant_bit() {
        let result = most_significant_bit(U256::zero());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Can not get most significant bit or least significant bit on zero value"
        );
    }

    fn test_least_significant_bit() {}
}
