use std::{ops::ShrAssign, str::FromStr};

use alloy::primitives::U256;

use crate::{error::UniswapV3MathError, U128_MAX, U16_MAX, U256_ONE, U32_MAX, U64_MAX, U8_MAX};

pub fn most_significant_bit(mut x: U256) -> Result<u8, UniswapV3MathError> {
    let mut r = 0;

    if x.is_zero() {
        return Err(UniswapV3MathError::ZeroValue);
    }

    if x >= U256::from_limbs([0, 0, 1, 0]) {
        x.shr_assign(128);
        r += 128;
    }

    if x >= U256::from_limbs([0, 1, 0, 0]) {
        x.shr_assign(64);
        r += 64;
    }

    if x >= U256::from_limbs([4294967296, 0, 0, 0]) {
        x.shr_assign(32);
        r += 32;
    }

    if x >= U256::from_limbs([65536, 0, 0, 0]) {
        x.shr_assign(16);
        r += 16;
    }

    if x >= U256::from_limbs([256, 0, 0, 0]) {
        x.shr_assign(8);
        r += 8;
    }

    if x >= U256::from_limbs([16, 0, 0, 0]) {
        x.shr_assign(4);
        r += 4;
    }
    if x >= U256::from_limbs([4, 0, 0, 0]) {
        x.shr_assign(2);
        r += 2;
    }

    if x >= U256::from_limbs([2, 0, 0, 0]) {
        r += 1;
    }

    Ok(r)
}

pub fn least_significant_bit(mut x: U256) -> Result<u8, UniswapV3MathError> {
    if x.is_zero() {
        return Err(UniswapV3MathError::ZeroValue);
    }

    let mut r = 255;

    if x & U128_MAX > U256::ZERO {
        r -= 128;
    } else {
        x >>= 128;
    }

    if x & U64_MAX > U256::ZERO {
        r -= 64;
    } else {
        x >>= 64;
    }

    if x & U32_MAX > U256::ZERO {
        r -= 32;
    } else {
        x >>= 32;
    }

    if x & U16_MAX > U256::ZERO {
        r -= 16;
    } else {
        x >>= 16;
    }

    if x & U8_MAX > U256::ZERO {
        r -= 8;
    } else {
        x >>= 8;
    }

    if x & U256::from_limbs([15, 0, 0, 0]) > U256::ZERO {
        r -= 4;
    } else {
        x >>= 4;
    }

    if x & U256::from_limbs([3, 0, 0, 0]) > U256::ZERO {
        r -= 2;
    } else {
        x >>= 2;
    }

    if x & U256_ONE > U256::ZERO {
        r -= 1;
    }

    Ok(r)
}

#[cfg(test)]
mod test {

    use std::str::FromStr;

    use alloy::primitives::U256;

    use crate::{bit_math::least_significant_bit, U256_ONE};

    use super::most_significant_bit;

    #[test]
    fn test_most_significant_bit() {
        //0
        let result = most_significant_bit(U256::ZERO);
        assert_eq!(
            result.unwrap_err().to_string(),
            "Can not get most significant bit or least significant bit on zero value"
        );

        //1
        let result = most_significant_bit(U256_ONE);
        assert_eq!(result.unwrap(), 0);

        //2
        let result = most_significant_bit(U256::from(2));
        assert_eq!(result.unwrap(), 1);

        //all powers of 2
        for i in 0..=255 {
            let result = most_significant_bit(U256::from(2).pow(U256::from(i)));
            assert_eq!(result.unwrap(), i as u8);
        }

        //uint256(-1)
        let result = most_significant_bit(
            //TODO:FIXME: might need to be from dec string
            U256::from_str(
                "115792089237316195423570985008687907853269984665640564039457584007913129639935",
            )
            .unwrap(),
        );
        assert_eq!(result.unwrap(), 255);
    }

    #[test]
    fn test_least_significant_bit() {
        //0
        let result = least_significant_bit(U256::ZERO);
        assert_eq!(
            result.unwrap_err().to_string(),
            "Can not get most significant bit or least significant bit on zero value"
        );

        //1
        let result = least_significant_bit(U256_ONE);
        assert_eq!(result.unwrap(), 0);

        //2
        let result = least_significant_bit(U256::from(2));
        assert_eq!(result.unwrap(), 1);

        //all powers of 2
        for i in 0..=255 {
            let result = least_significant_bit(U256::from(2).pow(U256::from(i)));
            assert_eq!(result.unwrap(), i as u8);
        }

        //uint256(-1)
        let result = least_significant_bit(
            //TODO:FIXME: might need to be from dec string
            U256::from_str(
                "115792089237316195423570985008687907853269984665640564039457584007913129639935",
            )
            .unwrap(),
        );
        assert_eq!(result.unwrap(), 0);
    }
}
