use alloy::primitives::{Uint, U256};

const ONE: U256 = U256::from_limbs([0, 0, 0, 1]);

pub fn div_rounding_up(a: U256, b: U256) -> U256 {
    let (quotient, remainder) = a.div_mod(b);
    if remainder.is_zero() {
        quotient
    } else {
        quotient + ONE
    }
}
