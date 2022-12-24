use ethers::types::U256;

pub fn div_rounding_up(a: U256, b: U256) -> U256 {
    let (quotent, remainder) = a.div_mod(b);
    if remainder.is_zero() {
        quotent
    } else {
        quotent + 1
    }
}
