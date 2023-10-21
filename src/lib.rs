pub mod abi;
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
pub mod utils;

use error::UniswapV3MathError;

pub trait BitmapReader {
    fn word_at_pos(&self, pos: i16) -> Result<U256, UniswapV3MathError>;
}
