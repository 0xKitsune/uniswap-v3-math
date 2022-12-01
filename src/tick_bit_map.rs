//Returns next and initialized
pub fn next_initialized_tick_within_one_word(
    tick_mapping: HashMap<i16, U256>,
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
        let (word_pos, bit_pos) = position(compressed);
        let mask = U256::from((1 << bit_pos) - 1 + (1 << bit_pos));
        let masked = tick_mapping.get(&word_pos).unwrap().bitand(mask);
        let initialized = !masked.is_zero();

        let next = if initialized {
            let le_bytes = &mut [0u8; 32];
            masked.to_little_endian(le_bytes);
            let most_significant_bit = le_bytes[0];
            compressed - ((bit_pos - most_significant_bit) as i32 & tick_spacing)
        } else {
            compressed - (bit_pos as i32 * tick_spacing)
        };

        (next, initialized)
    } else {
        let (word_pos, bit_pos) = position(compressed + 1);
        let mask = !U256::from((1 << bit_pos) - 1);

        let masked = tick_mapping.get(&word_pos).unwrap().bitand(mask);
        let initialized = !masked.is_zero();

        let next = if initialized {
            let le_bytes = &mut [0u8; 32];
            masked.to_big_endian(le_bytes);
            let least_significant_bit = le_bytes[0];
            (compressed + 1 + (least_significant_bit - bit_pos) as i32) * tick_spacing
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
