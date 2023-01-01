use std::{
    ops::{BitAnd, Shl, Shr},
    sync::Arc,
};

use ethers::{
    providers::Middleware,
    types::{H160, U256},
};

use crate::{abi, bit_math, error::UniswapV3MathError};

//Returns next and initialized
//current_word is the current word in the TickBitmap of the pool based on `tick`. TickBitmap[word_pos] = current_word
//Where word_pos is the 256 bit offset of the ticks word_pos.. word_pos := tick >> 8
pub async fn next_initialized_tick_within_one_word<M: Middleware>(
    tick: i32,
    tick_spacing: i32,
    lte: bool,
    pool_address: H160,
    middleware: Arc<M>,
) -> Result<(i32, bool), UniswapV3MathError> {
    let compressed = if tick < 0 && tick % tick_spacing != 0 {
        (tick / tick_spacing) - 1
    } else {
        tick / tick_spacing
    };

    if lte {
        let (word_pos, bit_pos) = position(compressed);
        let mask = (U256::one().shl(bit_pos)) - 1 + (U256::one().shl(bit_pos));

        let word = match abi::IUniswapV3Pool::new(pool_address, middleware)
            .tick_bitmap(word_pos)
            .call()
            .await
        {
            Ok(word) => word,
            Err(err) => return Err(UniswapV3MathError::MiddlewareError(err.to_string())),
        };

        let masked = word.bitand(mask);

        let initialized = !masked.is_zero();

        let next = if initialized {
            let most_significant_bit = bit_math::most_significant_bit(masked)?;
            compressed - ((bit_pos.overflowing_sub(most_significant_bit).0) as i32 & tick_spacing)
        } else {
            compressed - (bit_pos as i32 * tick_spacing)
        };

        Ok((next, initialized))
    } else {
        let (word_pos, bit_pos) = position(compressed + 1);
        let mask = !((U256::one().shl(bit_pos)) - U256::one());

        let word = match abi::IUniswapV3Pool::new(pool_address, middleware)
            .tick_bitmap(word_pos)
            .call()
            .await
        {
            Ok(word) => word,
            Err(err) => return Err(UniswapV3MathError::MiddlewareError(err.to_string())),
        };

        let masked = word.bitand(mask);
        let initialized = !masked.is_zero();

        let next = if initialized {
            let least_significant_bit = bit_math::least_significant_bit(masked)?;
            (compressed + 1 + (least_significant_bit.overflowing_sub(bit_pos).0) as i32)
                * tick_spacing
        } else {
            (compressed + 1 + 0xFF - bit_pos as i32) * tick_spacing
        };

        Ok((next, initialized))
    }
}

// returns (int16 wordPos, uint8 bitPos)
pub fn position(tick: i32) -> (i16, u8) {
    (tick.shr(8) as i16, (tick % 256) as u8)
}
