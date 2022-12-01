// returns (uint160 sqrtQX96)
pub fn get_next_sqrt_price_from_input(
    sqrt_price: U256,
    liquidity: u128,
    amount_in: U256,
    zero_for_one: bool,
) -> (U256) {
    //TODO: update this
    U256::zero()
}

// returns (uint160 sqrtQX96)
pub fn get_next_sqrt_price_From_amount_0_rounding_up(
    sqrt_price_x_96: U256,
    liquidity: u128,
    amount: U256,
    add: bool,
) -> U256 {
    if amount.is_zero() {
        return sqrt_price_x_96;
    }

    let numerator_1 = U256::from(liquidity).shl(96);

    if add {
        let product = amount * sqrt_price_x_96;

        if product / amount == sqrt_price_x_96 {
            let denominator = numerator_1.add(product);

            if denominator >= numerator_1 {
                return;
            }
        }
    }

    //TODO: update this
    U256::zero()
}

// returns (uint160 sqrtQX96)
pub fn get_next_sqrt_price_From_amount_1_rounding_down(
    sqrt_price_x_96: U256,
    liquidity: u128,
    amount: U256,
    add: bool,
) -> U256 {
    if add {
        let quotent = if amount <= 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF {
            amount.shl(96) / liquidity
        } else {
            FullMath::mul_div(amount, 0x1000000000000000000000000, liquidity)
        };

        (sqrt_price_x_96 +quotent) 
    } else {
        let quotent = if amount <= 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF {
            UnsafeMath::div_rounding_up(amount.shl(96), liquidity)
        } else {
            FullMath::mul_div_rounding_up(amount, 0x1000000000000000000000000, liquidity)
        };


        (sqrt_price_x_96 - quotent)
    }
   
}

// returns (uint256 amount0)
pub fn get_amount_0_delta(
    sqrt_ratio_a_x_96: U256,
    sqrt_ratio_b_x_96: U256,
    liquidity: u128,
    round_up: bool,
) -> U256 {
    //TODO: update this
    U256::zero()
}

// returns (uint256 amount1)
pub fn get_amount_1_delta(
    sqrt_ratio_a_x_96: U256,
    sqrt_ratio_b_x_96: U256,
    liquidity: u128,
    round_up: bool,
) -> U256 {
    //TODO: update this
    U256::zero()
}
