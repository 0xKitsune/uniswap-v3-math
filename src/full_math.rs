// returns (uint256 result)
fn mul_div(a: U256, b: U256, denominator: U256) -> U256 {
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
