use crate::{abi, bit_math, error::UniswapV3MathError};
use ethers::{
    providers::Middleware,
    types::{BlockNumber, H160, U256},
};
use std::{collections::HashMap, sync::Arc};

pub fn flip_tick(
    tick_bitmap: &mut HashMap<i16, U256>,
    tick: i32,
    tick_spacing: i32,
) -> Result<(), UniswapV3MathError> {
    if (tick % tick_spacing) != 0 {
        return Err(UniswapV3MathError::TickSpacingError);
    }

    let (word_pos, bit_pos) = position(tick / tick_spacing);
    let mask = U256::one() << bit_pos;
    let binding = U256::zero();
    let word = tick_bitmap.get(&word_pos).unwrap_or(&binding);
    tick_bitmap.insert(word_pos, *word ^ mask);
    Ok(())
}
//Returns next and initialized
//current_word is the current word in the TickBitmap of the pool based on `tick`. TickBitmap[word_pos] = current_word
//Where word_pos is the 256 bit offset of the ticks word_pos.. word_pos := tick >> 8
pub fn next_initialized_tick_within_one_word(
    tick_bitmap: &HashMap<i16, U256>,
    tick: i32,
    tick_spacing: i32,
    lte: bool,
) -> Result<(i32, bool), UniswapV3MathError> {
    let compressed = tick / tick_spacing;

    let (word_pos, bit_pos) = position(compressed);

    if lte {
        let mask = (U256::one() << bit_pos) - 1 + (U256::one() << bit_pos);

        let masked = *tick_bitmap.get(&word_pos).unwrap_or(&U256::zero()) & mask;

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

        let masked = tick_bitmap[&word_pos] & mask;

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

#[cfg(test)]
mod test {
    use std::{collections::HashMap, vec};

    use ethers::types::U256;

    use super::{flip_tick, next_initialized_tick_within_one_word};

    pub fn init_test_ticks() -> eyre::Result<HashMap<i16, U256>> {
        let test_ticks = vec![-200, -55, -4, 70, 78, 84, 139, 240, 535];
        let mut tick_bitmap: HashMap<i16, U256> = HashMap::new();
        for tick in test_ticks {
            flip_tick(&mut tick_bitmap, tick, 1)?;
        }
        Ok(tick_bitmap)
    }

    pub fn initialized(tick: i32, tick_bitmap: &HashMap<i16, U256>) -> eyre::Result<bool> {
        let (next, initialized) =
            next_initialized_tick_within_one_word(tick_bitmap, tick, 1, true)?;
        if next == tick {
            Ok(initialized)
        } else {
            Ok(false)
        }
    }
    #[test]
    pub fn test_next_initialized_tick_within_one_word() -> eyre::Result<()> {
        let tick_bitmap = init_test_ticks()?;
        //returns tick to right if at initialized tick
        let (next, initialized) =
            next_initialized_tick_within_one_word(&tick_bitmap, 78, 1, false)?;
        // vec![-200, -55, -4, 70, 78, 84, 139, 240, 535];
        // assert_eq!(next, 84);
        // assert_eq!(initialized, true);

        // //returns the tick directly to the right
        // let (next, initialized) =
        //     next_initialized_tick_within_one_word(&tick_bitmap, 77, 1, false)?;

        // assert_eq!(next, 78);
        // assert_eq!(initialized, true);

        // //returns the tick directly to the right
        // let (next, initialized) =
        //     next_initialized_tick_within_one_word(&tick_bitmap, -56, 1, false)?;

        // assert_eq!(next, -55);
        // assert_eq!(initialized, true);
        // //returns the next words initialized tick if on the right boundary
        // let (next, initialized) =
        //     next_initialized_tick_within_one_word(&tick_bitmap, 255, 1, false)?;

        // assert_eq!(next, 511);
        // assert_eq!(initialized, false);
        // //returns the next words initialized tick if on the right boundary
        // let (next, initialized) =
        //     next_initialized_tick_within_one_word(&tick_bitmap, -257, 1, false)?;

        // assert_eq!(next, -200);
        // assert_eq!(initialized, true);
        // //returns the next initialized tick from the next word
        // flip_tick(&mut tick_bitmap, 340, 1)?;
        // let (next, initialized) =
        //     next_initialized_tick_within_one_word(&tick_bitmap, 328, 1, false)?;

        // assert_eq!(next, 340);
        // assert_eq!(initialized, true);
        // //does not exceed boundary
        // let (next, initialized) =
        //     next_initialized_tick_within_one_word(&tick_bitmap, 508, 1, false)?;

        // assert_eq!(next, 511);
        // assert_eq!(initialized, false);
        // //skips entire word
        // let (next, initialized) =
        //     next_initialized_tick_within_one_word(&tick_bitmap, 255, 1, false)?;

        // assert_eq!(next, 511);
        // assert_eq!(initialized, false);
        // //skips half word
        // let (next, initialized) =
        //     next_initialized_tick_within_one_word(&tick_bitmap, 383, 1, false)?;

        // assert_eq!(next, 511);
        // assert_eq!(initialized, false);
        Ok(())
    }

    #[test]
    pub fn test_initialized_0() -> eyre::Result<()> {
        //is false at first
        let mut tick_bitmap: HashMap<i16, U256> = HashMap::new();
        let initialized_0 = initialized(1, &tick_bitmap)?;

        assert_eq!(initialized_0, false);
        //is flipped by #flipTick
        flip_tick(&mut tick_bitmap, 1, 1)?;
        let initialized_1 = initialized(1, &tick_bitmap)?;
        assert_eq!(initialized_1, true);

        Ok(())
    }
    #[test]
    pub fn test_initialized_1() -> eyre::Result<()> {
        let mut tick_bitmap: HashMap<i16, U256> = HashMap::new();
        //is flipped back by #flipTick
        flip_tick(&mut tick_bitmap, 1, 1)?;
        flip_tick(&mut tick_bitmap, 1, 1)?;
        let initialized_2 = initialized(1, &tick_bitmap)?;
        assert_eq!(initialized_2, false);
        Ok(())
    }

    #[test]
    pub fn test_initialized_2() -> eyre::Result<()> {
        //is not changed by another flip to a different tick
        let mut tick_bitmap: HashMap<i16, U256> = HashMap::new();
        flip_tick(&mut tick_bitmap, 2, 1)?;
        let initialized = initialized(1, &tick_bitmap)?;
        assert_eq!(initialized, false);
        Ok(())
    }

    #[test]
    pub fn test_initialized_3() -> eyre::Result<()> {
        //is not changed by another flip to a different tick on another word
        let mut tick_bitmap: HashMap<i16, U256> = HashMap::new();
        flip_tick(&mut tick_bitmap, 1 + 256, 1)?;
        let initialized_0 = initialized(257, &tick_bitmap)?;
        assert_eq!(initialized_0, true);
        let initialized_1 = initialized(1, &tick_bitmap)?;
        assert_eq!(initialized_1, false);
        Ok(())
    }
    #[test]
    pub fn test_flip_tick() -> eyre::Result<()> {
        let mut tick_bitmap = HashMap::new();
        flip_tick(&mut tick_bitmap, -230, 1)?;
        let initialized_0 = initialized(-230, &tick_bitmap)?;
        assert_eq!(initialized_0, true);
        let initialized_1 = initialized(-231, &tick_bitmap)?;
        assert_eq!(initialized_1, false);
        let initialized_2 = initialized(-229, &tick_bitmap)?;
        assert_eq!(initialized_2, false);
        let initialized_3 = initialized(-230 + 256, &tick_bitmap)?;
        assert_eq!(initialized_3, false);
        let initialized_4 = initialized(-230 - 256, &tick_bitmap)?;
        assert_eq!(initialized_4, false);
        flip_tick(&mut tick_bitmap, -230, 1)?;
        let initialized_5 = initialized(-230, &tick_bitmap)?;
        assert_eq!(initialized_5, false);
        let initialized_6 = initialized(-231, &tick_bitmap)?;
        assert_eq!(initialized_6, false);
        let initialized_7 = initialized(-229, &tick_bitmap)?;
        assert_eq!(initialized_7, false);
        let initialized_8 = initialized(-230 + 256, &tick_bitmap)?;
        assert_eq!(initialized_8, false);
        let initialized_9 = initialized(-230 - 256, &tick_bitmap)?;
        assert_eq!(initialized_9, false);

        Ok(())
    }
}
