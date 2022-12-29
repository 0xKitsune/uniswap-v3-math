use thiserror::Error;

#[derive(Error, Debug)]

//TODO: make these errors better, some errors in univ3 libs are just require(condition) without a message.
pub enum UniswapV3MathError {
    #[error("Denominator is 0")]
    DenominatorIsZero(),
    #[error("Result is U256::MAX")]
    ResultIsU256MAX(),
    #[error("Sqrt price is 0")]
    SqrtPriceIsZero(),
    #[error("Liquidity is 0")]
    LiquidityIsZero(),
    //TODO: Update this, shield your eyes for now
    #[error(
        "require((product = amount * sqrtPX96) / amount == sqrtPX96 && numerator1 > product);"
    )]
    ProductDivAmount(),
    #[error("Denominator is less than or equal to prod_1")]
    DenominatorIsLteProdOne(),
    #[error("Liquidity Sub")]
    LiquiditySub(),
    #[error("Liquidity Add")]
    LiquidityAdd(),
    #[error("The given tick must be less than, or equal to, the maximum tick")]
    T(),
    #[error(
        "Second inequality must be < because the price can never reach the price at the max tick"
    )]
    R(),
    #[error("Middleware error when getting next_initialized_tick_within_one_word")]
    MiddlewareError(String),
}
