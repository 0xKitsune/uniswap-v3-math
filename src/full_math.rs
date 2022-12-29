use ethers::types::U256;
use ruint::Uint;
use std::ops::{BitAnd, BitOrAssign};

use crate::error::UniswapV3MathError;

pub fn mul_mod(a: U256, b: U256, denominator: U256) -> U256 {
    let mut a_le_bytes = [0_u8; 32];
    a.to_little_endian(&mut a_le_bytes);
    let a: Uint<256, 4> = ruint::Uint::from_le_bytes(a_le_bytes);

    let mut b_le_bytes = [0_u8; 32];
    b.to_little_endian(&mut b_le_bytes);
    let b: Uint<256, 4> = ruint::Uint::from_le_bytes(b_le_bytes);

    let mut d_le_bytes = [0_u8; 32];
    denominator.to_little_endian(&mut d_le_bytes);
    let d: Uint<256, 4> = ruint::Uint::from_le_bytes(d_le_bytes);

    let result: [u8; 32] = a.mul_mod(b, d).to_le_bytes();

    U256::from_little_endian(&result)
}

// returns (uint256 result)
pub fn mul_div(a: U256, b: U256, mut denominator: U256) -> Result<U256, UniswapV3MathError> {
    // 512-bit multiply [prod1 prod0] = a * b
    // Compute the product mod 2**256 and mod 2**256 - 1
    // then use the Chinese Remainder Theorem to reconstruct
    // the 512 bit result. The result is stored in two 256
    // variables such that product = prod1 * 2**256 + prod0
    let mm = mul_mod(a, b, U256::MAX);
    let mut prod_0 = a.overflowing_mul(b).0; // Least significant 256 bits of the product
    let mut prod_1 = mm
        .overflowing_sub(prod_0)
        .0
        .overflowing_sub(U256::from((mm < prod_0) as u8))
        .0;

    // Handle non-overflow cases, 256 by 256 division
    if prod_1.is_zero() {
        if denominator == U256::zero() {
            return Err(UniswapV3MathError::DenominatorIsZero());
        }
        return Ok(prod_0 / denominator);
    }

    // Make sure the result is less than 2**256.
    // Also prevents denominator == 0
    if denominator <= prod_1 {
        return Err(UniswapV3MathError::DenominatorIsLteProdOne());
    }

    ///////////////////////////////////////////////
    // 512 by 256 division.
    ///////////////////////////////////////////////
    //

    // Make division exact by subtracting the remainder from [prod1 prod0]
    // Compute remainder using mulmod
    let remainder = mul_mod(a, b, denominator);

    // Subtract 256 bit number from 512 bit number
    prod_1 = prod_1
        .overflowing_sub(U256::from((remainder > prod_0) as u8))
        .0;
    prod_0 = prod_0.overflowing_sub(remainder).0;

    // Factor powers of two out of denominator
    // Compute largest power of two divisor of denominator.
    // Always >= 1.
    let mut twos = U256::zero()
        .overflowing_sub(denominator)
        .0
        .bitand(denominator);

    // Divide denominator by power of two
    //TODO: this is in an assembly block, this should be able to underflow
    denominator /= twos;

    // Divide [prod1 prod0] by the factors of two
    //TODO: this is in an assembly block, this should be able to underflow

    prod_0 /= twos;

    // Shift in bits from prod1 into prod0. For this we need
    // to flip `twos` such that it is 2**256 / twos.
    // If twos is zero, then it becomes one
    //TODO: this is in an assembly block, this should be able to underflow
    twos = (U256::zero().overflowing_sub(twos).0 / twos) + U256::one();

    prod_0.bitor_assign(prod_1 * twos);

    // Invert denominator mod 2**256
    // Now that denominator is an odd number, it has an inverse
    // modulo 2**256 such that denominator * inv = 1 mod 2**256.
    // Compute the inverse by starting with a seed that is correct
    // correct for four bits. That is, denominator * inv = 1 mod 2**4

    let mut inv = (U256::from(3) * denominator) * (U256::from(3) * denominator);

    // Now use Newton-Raphson iteration to improve the precision.
    // Thanks to Hensel's lifting lemma, this also works in modular
    // arithmetic, doubling the correct bits in each step.
    inv *= U256::from(2) - denominator * inv;
    inv *= U256::from(2) - denominator * inv; // inverse mod 2**8
    inv *= U256::from(2) - denominator * inv; // inverse mod 2**16
    inv *= U256::from(2) - denominator * inv; // inverse mod 2**32
    inv *= U256::from(2) - denominator * inv; // inverse mod 2**64
    inv *= U256::from(2) - denominator * inv; // inverse mod 2**128
    inv *= U256::from(2) - denominator * inv; // inverse mod 2**256

    // Because the division is now exact we can divide by multiplying
    // with the modular inverse of denominator. This will give us the
    // correct result modulo 2**256. Since the precoditions guarantee
    // that the outcome is less than 2**256, this is the final result.
    // We don't need to compute the high bits of the result and prod1
    // is no longer required.
    Ok(prod_0 * inv)
}

pub fn mul_div_rounding_up(
    a: U256,
    b: U256,
    denominator: U256,
) -> Result<U256, UniswapV3MathError> {
    let result = mul_div(a, b, denominator)?;

    if mul_mod(a, b, denominator) > U256::zero() {
        if result == U256::MAX {
            return Err(UniswapV3MathError::ResultIsU256MAX());
        } else {
            return Ok(result + 1);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use ethers::types::U256;

    use crate::full_math::mul_mod;

    use super::mul_div;

    const Q128: U256 = U256([0, 0, 1, 0]);

    #[test]
    fn test_mul_mod() {
        let result = mul_mod(U256::from(256), U256::from(5), U256::MAX);
        assert_eq!(result, U256::from(1280));

        let result = mul_mod(U256::from(100), U256::from(100), U256::from(21));
        assert_eq!(result, U256::from(4));

        let result = mul_mod(U256::from(100), U256::from(100), U256::from(0));
        assert_eq!(result, U256::from(0));
    }

    #[test]
    fn test_mul_div() {
        //Revert if the denominator is zero
        let result = mul_div(Q128, U256::from(5), U256::zero());
        assert_eq!(result.err().unwrap().to_string(), "Denominator is 0");

        // Revert if the denominator is zero and numerator overflows
        let result = mul_div(Q128, Q128, U256::zero());
        assert_eq!(
            result.err().unwrap().to_string(),
            "Denominator is less than or equal to prod_1"
        );

        // Revert if the output overflows uint256
        let result = mul_div(Q128, Q128, U256::one());
        assert_eq!(
            result.err().unwrap().to_string(),
            "Denominator is less than or equal to prod_1"
        );
    }
}
