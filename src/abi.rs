sol! {
    #[sol(rpc)]
    contract UniswapV3Pool {

        #[derive(Debug)]
        function tickBitmap(int16) external returns (uint256);
    }
}
