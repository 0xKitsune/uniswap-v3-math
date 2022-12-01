// returns (uint128 z)
pub fn add_delta(x: u128, y: i128) -> u128 {
    if y < 0 {
        let z = x - (-y as u128);

        if z < x {
            //TODO: revert "LS" error
            0 // right now just zero to avoid linting error
        } else {
            z
        }
    } else {
        let z = x - (y as u128);
        if z >= x {
            //TODO: revert "LA" error
            0 // right now just zero to avoid linting error
        } else {
            z
        }
    }
}
