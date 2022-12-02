use std::ops::Shl;

use ethers::types::{I256, U256};

use crate::{
    full_math::{mul_div, mul_div_rounding_up},
    unsafe_math::div_rounding_up,
};

// returns (uint160 sqrtQX96)
pub fn get_next_sqrt_price_from_input(
    sqrt_price: U256,
    liquidity: u128,
    amount_in: U256,
    zero_for_one: bool,
) -> U256 {
    //TODO: require sqrtPriceX96 > 0
    //TODO: require liquidity > 0
    if zero_for_one {
        get_next_sqrt_price_From_amount_0_rounding_up(sqrt_price, liquidity, amount_in, true)
    } else {
        get_next_sqrt_price_From_amount_1_rounding_down(sqrt_price, liquidity, amount_in, true)
    }
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
            let denominator = numerator_1 + product;

            if denominator >= numerator_1 {
                return mul_div_rounding_up(numerator_1, sqrt_price_x_96, denominator);
            }
        }

        return div_rounding_up(numerator_1, (numerator_1 / sqrt_price_x_96) + amount);
    } else {
        let product = amount * sqrt_price_x_96;
        if product / amount == sqrt_price_x_96 && (numerator_1 > product) {
            let denominator = numerator_1 - product;

            return mul_div_rounding_up(numerator_1, sqrt_price_x_96, denominator);
        } else {
            return U256::from(0);
        }
    }
}

// returns (uint160 sqrtQX96)
pub fn get_next_sqrt_price_From_amount_1_rounding_down(
    sqrt_price_x_96: U256,
    liquidity: u128,
    amount: U256,
    add: bool,
) -> U256 {
    if add {
        let quotent = if amount <= U256::from("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF") {
            amount.shl(96) / liquidity
        } else {
            mul_div(
                amount,
                U256::from("0x1000000000000000000000000"),
                U256::from(liquidity),
            )
        };

        sqrt_price_x_96 + quotent
    } else {
        let quotent = if amount <= U256::from("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF") {
            div_rounding_up(amount.shl(96), U256::from(liquidity))
        } else {
            mul_div_rounding_up(
                amount,
                U256::from("0x1000000000000000000000000"),
                U256::from(liquidity),
            )
        };

        sqrt_price_x_96 - quotent
    }
}

// returns (uint256 amount0)
pub fn _get_amount_0_delta(
    sqrt_ratio_a_x_96: U256,
    sqrt_ratio_b_x_96: U256,
    liquidity: i128,
    round_up: bool,
) -> U256 {
    let (sqrt_ratio_a_x_96, sqrt_ratio_b_x_96) = if sqrt_ratio_a_x_96 > sqrt_ratio_b_x_96 {
        (sqrt_ratio_a_x_96, sqrt_ratio_b_x_96)
    } else {
        (sqrt_ratio_b_x_96, sqrt_ratio_a_x_96)
    };

    let numerator_1 = U256::from(liquidity).shl(96);
    let numerator_2 = sqrt_ratio_a_x_96 - sqrt_ratio_b_x_96;

    //TODO: Add require check tyhat sqrtRatioAX96 > 0
    if round_up {
        let numerator_partial = mul_div_rounding_up(numerator_1, numerator_2, sqrt_ratio_b_x_96);
        return div_rounding_up(numerator_partial, sqrt_ratio_a_x_96);
    } else {
        return mul_div(numerator_1, numerator_2, sqrt_ratio_b_x_96) / sqrt_ratio_a_x_96;
    };
}

// returns (uint256 amount1)
pub fn _get_amount_1_delta(
    mut sqrt_ratio_a_x_96: U256,
    mut sqrt_ratio_b_x_96: U256,
    liquidity: i128,
    round_up: bool,
) -> U256 {
    (sqrt_ratio_a_x_96, sqrt_ratio_b_x_96) = if sqrt_ratio_a_x_96 > sqrt_ratio_b_x_96 {
        (sqrt_ratio_a_x_96, sqrt_ratio_b_x_96)
    } else {
        (sqrt_ratio_b_x_96, sqrt_ratio_a_x_96)
    };

    if round_up {
        return mul_div_rounding_up(
            U256::from(liquidity),
            sqrt_ratio_b_x_96 - sqrt_ratio_a_x_96,
            U256::from("0x1000000000000000000000000"),
        );
    } else {
        return mul_div(
            U256::from(liquidity),
            sqrt_ratio_b_x_96 - sqrt_ratio_a_x_96,
            U256::from("0x1000000000000000000000000"),
        );
    }
}

pub fn get_amount_0_delta(
    sqrt_ratio_a_x_96: U256,
    sqrt_ratio_b_x_96: U256,
    liquidity: i128,
) -> I256 {
    if liquidity < 0 {
        return I256::from_raw(_get_amount_0_delta(
            sqrt_ratio_b_x_96,
            sqrt_ratio_a_x_96,
            -liquidity as i128,
            false,
        ));
    } else {
        return I256::from_raw(_get_amount_0_delta(
            sqrt_ratio_a_x_96,
            sqrt_ratio_b_x_96,
            liquidity,
            true,
        ));
    }
}

pub fn get_amount_1_delta(
    sqrt_ratio_a_x_96: U256,
    sqrt_ratio_b_x_96: U256,
    liquidity: i128,
) -> I256 {
    if liquidity < 0 {
        return I256::from_raw(_get_amount_1_delta(
            sqrt_ratio_b_x_96,
            sqrt_ratio_a_x_96,
            -liquidity as i128,
            false,
        ));
    } else {
        return I256::from_raw(_get_amount_1_delta(
            sqrt_ratio_a_x_96,
            sqrt_ratio_b_x_96,
            liquidity,
            true,
        ));
    }
}
