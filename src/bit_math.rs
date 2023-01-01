use std::ops::ShrAssign;

use ethers::types::U256;

use crate::error::UniswapV3MathError;

pub fn most_significant_bit(mut x: U256) -> Result<u8, UniswapV3MathError> {
    let mut r = 0;

    if x.is_zero() {
        return Err(UniswapV3MathError::ZeroValue());
    }

    if x >= U256::from("0x100000000000000000000000000000000") {
        x.shr_assign(128);
        r += 128;
    }

    if x >= U256::from("0x10000000000000000") {
        x.shr_assign(64);
        r += 64;
    }

    if x >= U256::from("0x100000000") {
        x.shr_assign(32);
        r += 32;
    }

    if x >= U256::from("0x10000") {
        x.shr_assign(16);
        r += 16;
    }

    if x >= U256::from("0x100") {
        x.shr_assign(8);
        r += 8;
    }

    if x >= U256::from("0x10") {
        x.shr_assign(4);
        r += 4;
    }
    if x >= U256::from("0x4") {
        x.shr_assign(2);
        r += 2;
    }

    if x >= U256::from("0x2") {
        r += 1;
    }

    Ok(r)
}

pub fn least_significant_bit(mut x: U256) -> Result<u8, UniswapV3MathError> {
    let mut r = 255;
    if x.is_zero() {
        return Err(UniswapV3MathError::ZeroValue());
    }

    if x.as_u128() & u128::MAX > 0 {
        r -= 128;
    } else {
        x.shr_assign(128);
    }

    if x.as_u64() & u64::MAX > 0 {
        r -= 64;
    } else {
        x.shr_assign(64);
    }

    if x.as_u32() & u32::MAX > 0 {
        r -= 32;
    } else {
        x.shr_assign(32);
    }

    if x.as_u32() as u16 & u16::MAX > 0 {
        r -= 16;
    } else {
        x.shr_assign(16);
    }

    if x.as_u32() as u8 & u8::MAX > 0 {
        r -= 8;
    } else {
        x.shr_assign(8);
    }

    if x & U256::from("0xf") > U256::zero() {
        r -= 4;
    } else {
        x.shr_assign(4);
    }

    if x & U256::from("0x3") > U256::zero() {
        r -= 2;
    } else {
        x.shr_assign(2);
    }

    if x & U256::from("0x1") > U256::zero() {
        r -= 1;
    }

    Ok(r)
}

#[cfg(test)]
mod test {

    use ethers::types::U256;

    use crate::error::UniswapV3MathError;

    use super::most_significant_bit;

    #[test]
    fn test_most_significant_bit() {
        //0
        let result = most_significant_bit(U256::zero());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Can not get most significant bit or least significant bit on zero value"
        );

        //1
        let result = most_significant_bit(U256::one());
        assert_eq!(result.unwrap(), 0);

        //2
        let result = most_significant_bit(U256::from(2));
        assert_eq!(result.unwrap(), 1);
    }

    fn test_least_significant_bit() {}
}
