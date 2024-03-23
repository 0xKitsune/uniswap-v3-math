use alloy::primitives::U256;

pub mod bit_math;
pub mod error;
pub mod full_math;
pub mod liquidity_math;
pub mod sqrt_price_math;
pub mod swap_math;
pub mod tick;
pub mod tick_bitmap;
pub mod tick_math;
pub mod unsafe_math;

const U256_ONE: U256 = U256::from_limbs([0, 0, 0, 1]);
const U256_TWO: U256 = U256::from_limbs([0, 0, 0, 2]);
