use std::ops::{BitAnd, Shr};

use ethers::types::U256;

//Returns next and initialized
//current_word is the current word in the TickBitmap of the pool based on `tick`. TickBitmap[word_pos] = current_word
//Where word_pos is the 256 bit offset of the ticks word_pos.. word_pos := tick >> 8
pub fn next_initialized_tick_within_one_word(
    current_word: U256,
    tick: i32,
    tick_spacing: i32,
    lte: bool,
) -> (i32, bool) {
    let compressed = if tick < 0 && tick % tick_spacing != 0 {
        (tick / tick_spacing) - 1
    } else {
        tick / tick_spacing
    };

    if lte {
        let bit_pos = position(compressed).1;
        let mask = U256::from((1 << bit_pos) - 1 + (1 << bit_pos));
        let masked = current_word.bitand(mask);
        let initialized = !masked.is_zero();

        let next = if initialized {
            let be_bytes = &mut [0u8; 32];
            masked.to_big_endian(be_bytes);
            let most_significant_bit = be_bytes[0];
            compressed - ((bit_pos.overflowing_sub(most_significant_bit).0) as i32 & tick_spacing)
        } else {
            compressed - (bit_pos as i32 * tick_spacing)
        };

        (next, initialized)
    } else {
        let bit_pos = position(compressed + 1).1;
        let mask = !U256::from((1 << bit_pos) - 1);

        let masked = current_word.bitand(mask);
        let initialized = !masked.is_zero();

        let next = if initialized {
            let le_bytes = &mut [0u8; 32];
            masked.to_little_endian(le_bytes);
            let least_significant_bit = le_bytes[0];
            (compressed + 1 + (least_significant_bit.overflowing_sub(bit_pos).0) as i32)
                * tick_spacing
        } else {
            (compressed + 1 + 0xFF - bit_pos as i32) * tick_spacing
        };

        (next, initialized)
    }
}

// returns (int16 wordPos, uint8 bitPos)
pub fn position(tick: i32) -> (i16, u8) {
    (tick.shr(8) as i16, (tick % 256) as u8)
}
