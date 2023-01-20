use crate::{abi, bit_math, error::UniswapV3MathError};
use ethers::{
    providers::Middleware,
    types::{BlockNumber, H160, U256},
};
use std::sync::Arc;

//Returns next and initialized
//current_word is the current word in the TickBitmap of the pool based on `tick`. TickBitmap[word_pos] = current_word
//Where word_pos is the 256 bit offset of the ticks word_pos.. word_pos := tick >> 8
pub async fn next_initialized_tick_within_one_word(
    tick_spacing: i32,
    lte: bool,
    compressed: i32,
    bit_pos: u8,
    word: U256,
) -> Result<(i32, bool), UniswapV3MathError> {
    if lte {
        let mask = (U256::one() << bit_pos) - 1 + (U256::one() << bit_pos);

        let masked = word & mask;

        let initialized = !masked.is_zero();

        let next = if initialized {
            (compressed
                - (bit_pos
                    .overflowing_sub(bit_math::most_significant_bit(masked)?)
                    .0) as i32)
                * tick_spacing
        } else {
            (compressed - bit_pos as i32) * tick_spacing
        };

        Ok((next, initialized))
    } else {
        let mask = !((U256::one() << bit_pos) - U256::one());

        let masked = word & mask;
        let initialized = !masked.is_zero();

        let next = if initialized {
            (compressed
                + 1
                + (bit_math::least_significant_bit(masked)?
                    .overflowing_sub(bit_pos)
                    .0) as i32)
                * tick_spacing
        } else {
            (compressed + 1 + ((0xFF - bit_pos) as i32)) * tick_spacing
        };

        Ok((next, initialized))
    }
}

//Returns next and initialized. This function calls the node to get the word at the word_pos.
//current_word is the current word in the TickBitmap of the pool based on `tick`. TickBitmap[word_pos] = current_word
//Where word_pos is the 256 bit offset of the ticks word_pos.. word_pos := tick >> 8
pub async fn next_initialized_tick_within_one_word_from_provider<M: Middleware>(
    tick: i32,
    tick_spacing: i32,
    lte: bool,
    pool_address: H160,
    block_number: Option<BlockNumber>,
    middleware: Arc<M>,
) -> Result<(i32, bool), UniswapV3MathError> {
    let compressed = if tick < 0 && tick % tick_spacing != 0 {
        (tick / tick_spacing) - 1
    } else {
        tick / tick_spacing
    };

    if lte {
        let (word_pos, bit_pos) = position(compressed);
        let mask = (U256::one() << bit_pos) - 1 + (U256::one() << bit_pos);

        let word: U256 = if block_number.is_some() {
            match abi::IUniswapV3Pool::new(pool_address, middleware)
                .tick_bitmap(word_pos)
                .block(block_number.unwrap())
                .call()
                .await
            {
                Ok(word) => word,
                Err(err) => return Err(UniswapV3MathError::MiddlewareError(err.to_string())),
            }
        } else {
            match abi::IUniswapV3Pool::new(pool_address, middleware)
                .tick_bitmap(word_pos)
                .call()
                .await
            {
                Ok(word) => word,
                Err(err) => return Err(UniswapV3MathError::MiddlewareError(err.to_string())),
            }
        };

        let masked = word & mask;

        let initialized = !masked.is_zero();

        let next = if initialized {
            (compressed
                - (bit_pos
                    .overflowing_sub(bit_math::most_significant_bit(masked)?)
                    .0) as i32)
                * tick_spacing
        } else {
            (compressed - bit_pos as i32) * tick_spacing
        };

        Ok((next, initialized))
    } else {
        let (word_pos, bit_pos) = position(compressed + 1);
        let mask = !((U256::one() << bit_pos) - U256::one());

        let word: U256 = if block_number.is_some() {
            match abi::IUniswapV3Pool::new(pool_address, middleware)
                .tick_bitmap(word_pos)
                .block(block_number.unwrap())
                .call()
                .await
            {
                Ok(word) => word,
                Err(err) => return Err(UniswapV3MathError::MiddlewareError(err.to_string())),
            }
        } else {
            match abi::IUniswapV3Pool::new(pool_address, middleware)
                .tick_bitmap(word_pos)
                .call()
                .await
            {
                Ok(word) => word,
                Err(err) => return Err(UniswapV3MathError::MiddlewareError(err.to_string())),
            }
        };

        let masked = word & mask;
        let initialized = !masked.is_zero();

        let next = if initialized {
            (compressed
                + 1
                + (bit_math::least_significant_bit(masked)?
                    .overflowing_sub(bit_pos)
                    .0) as i32)
                * tick_spacing
        } else {
            (compressed + 1 + ((0xFF - bit_pos) as i32)) * tick_spacing
        };

        Ok((next, initialized))
    }
}

// returns (int16 wordPos, uint8 bitPos)
pub fn position(tick: i32) -> (i16, u8) {
    ((tick >> 8) as i16, (tick % 256) as u8)
}
