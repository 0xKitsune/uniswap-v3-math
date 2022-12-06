use thiserror::Error;

#[derive(Error, Debug)]

//TODO: make these errors better, some errors in univ3 libs are just require(condition) without a message.
pub enum UniswapV3Error {
    #[error("Denominator is greater than 0")]
    DenominatorIsGreaterThanZero(),
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
}
