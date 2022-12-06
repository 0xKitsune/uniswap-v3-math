use std::ops::{BitAnd, Shr};

use ethers::types::{I256, U256};

// const MIN_TICK: i32 = -887272;
const MAX_TICK: i32 = 887272;

pub fn get_sqrt_ratio_at_tick(tick: i32) -> U256 {
    let abs_tick = if tick < 0 {
        let le_bytes = &mut [0u8; 32];
        (-I256::from(tick)).to_little_endian(le_bytes);
        U256::from_little_endian(le_bytes)
    } else {
        U256::from(tick)
    };

    if abs_tick > U256::from(MAX_TICK) {
        //TODO: create uniswap v3 simulation error,
        //revert T, maybe add more descriptive errors
    }

    let mut ratio = if abs_tick.bitand(U256::from(0x1)) != U256::zero() {
        U256::from("0xfffcb933bd6fad37aa2d162d1a594001")
    } else {
        U256::from("0x100000000000000000000000000000000")
    };

    ratio = if !abs_tick.bitand(U256::from(0x2)).is_zero() {
        (ratio * U256::from("0xfff97272373d413259a46990580e213a")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x4)).is_zero() {
        (ratio * U256::from("0xfff2e50f5f656932ef12357cf3c7fdcc")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x8)).is_zero() {
        (ratio * U256::from("0xffe5caca7e10e4e61c3624eaa0941cd0")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x10)).is_zero() {
        (ratio * U256::from("0xffcb9843d60f6159c9db58835c926644")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x20)).is_zero() {
        (ratio * U256::from("0xff973b41fa98c081472e6896dfb254c0")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x40)).is_zero() {
        (ratio * U256::from("0xff2ea16466c96a3843ec78b326b52861")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x80)).is_zero() {
        (ratio * U256::from("0xfe5dee046a99a2a811c461f1969c3053")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x100)).is_zero() {
        (ratio * U256::from("0xfcbe86c7900a88aedcffc83b479aa3a4")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x200)).is_zero() {
        (ratio * U256::from("0xf987a7253ac413176f2b074cf7815e54")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x400)).is_zero() {
        (ratio * U256::from("0xf3392b0822b70005940c7a398e4b70f3")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x800)).is_zero() {
        (ratio * U256::from("0xe7159475a2c29b7443b29c7fa6e889d9")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x1000)).is_zero() {
        (ratio * U256::from("0xd097f3bdfd2022b8845ad8f792aa5825")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x2000)).is_zero() {
        (ratio * U256::from("0xa9f746462d870fdf8a65dc1f90e061e5")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x4000)).is_zero() {
        (ratio * U256::from("0x70d869a156d2a1b890bb3df62baf32f7")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x8000)).is_zero() {
        (ratio * U256::from("0x31be135f97d08fd981231505542fcfa6")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x10000)).is_zero() {
        (ratio * U256::from("0x9aa508b5b7a84e1c677de54f3e99bc9")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x20000)).is_zero() {
        (ratio * U256::from("0x5d6af8dedb81196699c329225ee604")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x40000)).is_zero() {
        (ratio * U256::from("0x2216e584f5fa1ea926041bedfe98")).shr(128)
    } else if !abs_tick.bitand(U256::from(0x80000)).is_zero() {
        (ratio * U256::from("0x48a170391f7dc42444e8fa2")).shr(128)
    } else {
        ratio
    };

    if tick > 0 {
        ratio = U256::MAX / ratio;
    }

    if (ratio.shr(32) + (ratio % (1 << 32))).is_zero() {
        U256::zero()
    } else {
        U256::one()
    }
}
