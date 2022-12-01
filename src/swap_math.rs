// //returns (
//         uint160 sqrtRatioNextX96,
//         uint256 amountIn,
//         uint256 amountOut,
//         uint256 feeAmount
//     )
pub fn compute_swap_step(
    sqrt_ratio_current_x_96: U256,
    sqrt_ratio_target_x_96: U256,
    liquidity: u128,
    amount_remaining: I256,
    fee_pips: u32,
) -> (U256, U256, U256, U256) {
    //TODO: update this
    (U256::zero(), U256::zero(), U256::zero(), U256::zero())
}
