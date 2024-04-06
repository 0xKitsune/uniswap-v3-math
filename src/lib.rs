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

const U256_ONE: U256 = U256::from_limbs([1, 0, 0, 0]);
const U128_MAX: U256 = U256::from_limbs([u64::MAX, u64::MAX, 0, 0]);
const U64_MAX: U256 = U256::from_limbs([u64::MAX, 0, 0, 0]);
const U32_MAX: U256 = U256::from_limbs([u32::MAX as u64, 0, 0, 0]);
const U16_MAX: U256 = U256::from_limbs([u16::MAX as u64, 0, 0, 0]);
const U8_MAX: U256 = U256::from_limbs([u8::MAX as u64, 0, 0, 0]);
