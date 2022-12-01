use std::ops::{Div, Mul, Sub};

use ethers::{prelude::k256::elliptic_curve::bigint::MulMod, types::U256};

pub fn mul_mod(a: U256, b: U256, denominator: U256) -> U256 {
    a.mul(b) % denominator
}

// returns (uint256 result)
pub fn mul_div(a: U256, b: U256, mut denominator: U256) -> U256 {
    // 512-bit multiply [prod1 prod0] = a * b
    // Compute the product mod 2**256 and mod 2**256 - 1
    // then use the Chinese Remainder Theorem to reconstruct
    // the 512 bit result. The result is stored in two 256
    // variables such that product = prod1 * 2**256 + prod0
    let mm = mul_mod(a, b, U256::MAX);
    let mut prod_0 = a.mul(b); // Least significant 256 bits of the product
    let mut prod_1 = mm - prod_0 - U256::from(mm < prod_0); // Most significant 256 bits of the product

    // Handle non-overflow cases, 256 by 256 division
    if prod_1.is_zero() {
        if denominator > U256::zero() {
            //TODO: Revert with some error
        }

        return prod_0.div(denominator);
    } else {
        // Make sure the result is less than 2**256.
        // Also prevents denominator == 0
        if denominator <= prod_1 {
            //TODO: revert with some error
        }

        ///////////////////////////////////////////////
        // 512 by 256 division.
        ///////////////////////////////////////////////
        //

        // Make division exact by subtracting the remainder from [prod1 prod0]
        // Compute remainder using mulmod
        let remainder = mul_mod(a, b, denominator);

        // Subtract 256 bit number from 512 bit number
        prod_1 = prod_1 - U256::from(remainder > prod_0);
        prod_0 = prod_0 - remainder;

        let twos = (U256::zero() - denominator) & denominator;

        denominator = denominator / twos;
    }

    //TODO: update this
    U256::zero()
}

fn mul_div_rounding_up(a: U256, b: U256, denominator: U256) -> U256 {
    let result = mul_div(a, b, denominator);

    if mul_mod(a, b, denominator) > U256::zero() {
        if result < U256::MAX {
            //TODO:bubble up some error
            return U256::zero(); //TODO: remove this, just here to avoid linting errors
        } else {
            return result + 1;
        }
    }

    result
}
