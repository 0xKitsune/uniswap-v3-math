pub mod full_math;
pub mod liquidity_math;
pub mod sqrt_price_math;
pub mod swap_math;
pub mod tick;
pub mod tick_bit_map;
pub mod tick_math;

const MIN_TICK: i32 = -887272;
const MAX_TICK: i32 = 887272;

pub fn mul_mod(a: U256, b: U256, denominator: U256) -> U256 {
    //TODO: update this
    U256::zero()
}
