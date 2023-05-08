use ethers::types::U256;
use ruint::Uint;

pub const RUINT_ZERO: Uint<256, 4> = Uint::ZERO;
pub const RUINT_ONE: Uint<256, 4> = Uint::<256, 4>::from_limbs([1, 0, 0, 0]);
pub const RUINT_TWO: Uint<256, 4> = Uint::<256, 4>::from_limbs([2, 0, 0, 0]);
pub const RUINT_THREE: Uint<256, 4> = Uint::<256, 4>::from_limbs([3, 0, 0, 0]);
pub const RUINT_MAX_U256: Uint<256, 4> = Uint::<256, 4>::from_limbs([
    18446744073709551615,
    18446744073709551615,
    18446744073709551615,
    18446744073709551615,
]);

pub fn u256_to_ruint(u: U256) -> Uint<256, 4> {
    Uint::from_limbs(u.0)
}

pub fn ruint_to_u256(r: Uint<256, 4>) -> U256 {
    U256(r.into_limbs())
}
