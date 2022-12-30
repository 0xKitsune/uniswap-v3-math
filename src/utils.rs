use ethers::types::U256;
use ruint::Uint;

pub const RUINT_ZERO: ruint::Uint<256, 4> = ruint::Uint::<256, 4>::from_limbs([0_u64; 4]);
pub const RUINT_ONE: ruint::Uint<256, 4> = ruint::Uint::<256, 4>::from_limbs([1, 0, 0, 0]);
pub const RUINT_TWO: ruint::Uint<256, 4> = ruint::Uint::<256, 4>::from_limbs([2, 0, 0, 0]);
pub const RUINT_THREE: ruint::Uint<256, 4> = ruint::Uint::<256, 4>::from_limbs([3, 0, 0, 0]);
pub const RUINT_MAX_U256: ruint::Uint<256, 4> = ruint::Uint::<256, 4>::from_limbs([
    18446744073709551615,
    18446744073709551615,
    18446744073709551615,
    18446744073709551615,
]);

pub fn u256_to_ruint(u: U256) -> ruint::Uint<256, 4> {
    let mut le_bytes = [0_u8; 32];
    u.to_little_endian(&mut le_bytes);
    ruint::Uint::from_le_bytes(le_bytes)
}

pub fn ruint_to_u256(r: Uint<256, 4>) -> U256 {
    U256::from_little_endian(&r.as_le_bytes())
}
