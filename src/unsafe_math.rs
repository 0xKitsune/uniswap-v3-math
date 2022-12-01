pub fn div_rounding_up(a: U256, b: U256) -> U256 {
    let (quotent, remainder) = a.div_rem(&b);
    if remainder.is_zero() {
        quotent
    } else {
        quotent + remainder
    }
}
