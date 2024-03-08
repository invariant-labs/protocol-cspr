use odra::{contract_env, execution_error, OdraType};

#[derive(OdraType, Debug, PartialEq)]
pub enum InvariantError {
    NotAdmin,
    NotFeeReceiver,
    PoolAlreadyExist,
    PoolNotFound,
    TickAlreadyExist,
    InvalidTickIndexOrTickSpacing,
    PositionNotFound,
    TickNotFound,
    FeeTierNotFound,
    PoolKeyNotFound,
    AmountIsZero,
    WrongLimit,
    PriceLimitReached,
    NoGainSwap,
    InvalidTickSpacing,
    FeeTierAlreadyExist,
    PoolKeyAlreadyExist,
    UnauthorizedFeeReceiver,
    ZeroLiquidity,
    TransferError,
    TokensAreSame,
    AmountUnderMinimumAmountOut,
    InvalidFee,
    NotEmptyTickDeinitialization,
    InvalidInitTick,
    InvalidInitSqrtPrice,
}

execution_error! {
    pub enum InvariantErrorReturn {
        NotAdmin => 0,
        NotFeeReceiver => 1,
        PoolAlreadyExist => 2,
        PoolNotFound => 3,
        TickAlreadyExist => 4,
        InvalidTickIndexOrTickSpacing => 5,
        PositionNotFound => 6,
        TickNotFound => 7,
        FeeTierNotFound => 8,
        PoolKeyNotFound => 9,
        AmountIsZero => 10,
        WrongLimit => 11,
        PriceLimitReached => 12,
        NoGainSwap => 13,
        InvalidTickSpacing => 14,
        FeeTierAlreadyExist => 15,
        PoolKeyAlreadyExist => 16,
        UnauthorizedFeeReceiver => 17,
        ZeroLiquidity => 18,
        TransferError => 19,
        TokensAreSame => 20,
        AmountUnderMinimumAmountOut => 21,
        InvalidFee => 22,
        NotEmptyTickDeinitialization => 23,
        InvalidInitTick => 24,
        InvalidInitSqrtPrice => 25,
    }
}

pub fn unwrap_invariant_result<T>(invariant_error: Result<T, InvariantError>) -> T {
    match invariant_error {
        Ok(result) => result,
        Err(invariant_error) => match invariant_error {
            InvariantError::NotAdmin => contract_env::revert(InvariantErrorReturn::NotAdmin),
            InvariantError::NotFeeReceiver => {
                contract_env::revert(InvariantErrorReturn::NotFeeReceiver)
            }
            InvariantError::PoolAlreadyExist => {
                contract_env::revert(InvariantErrorReturn::PoolAlreadyExist)
            }
            InvariantError::PoolNotFound => {
                contract_env::revert(InvariantErrorReturn::PoolNotFound)
            }
            InvariantError::TickAlreadyExist => {
                contract_env::revert(InvariantErrorReturn::TickAlreadyExist)
            }
            InvariantError::InvalidTickIndexOrTickSpacing => {
                contract_env::revert(InvariantErrorReturn::InvalidTickIndexOrTickSpacing)
            }
            InvariantError::PositionNotFound => {
                contract_env::revert(InvariantErrorReturn::PositionNotFound)
            }
            InvariantError::TickNotFound => {
                contract_env::revert(InvariantErrorReturn::TickNotFound)
            }
            InvariantError::FeeTierNotFound => {
                contract_env::revert(InvariantErrorReturn::FeeTierNotFound)
            }
            InvariantError::PoolKeyNotFound => {
                contract_env::revert(InvariantErrorReturn::PoolKeyNotFound)
            }
            InvariantError::AmountIsZero => {
                contract_env::revert(InvariantErrorReturn::AmountIsZero)
            }
            InvariantError::WrongLimit => contract_env::revert(InvariantErrorReturn::WrongLimit),
            InvariantError::PriceLimitReached => {
                contract_env::revert(InvariantErrorReturn::PriceLimitReached)
            }
            InvariantError::NoGainSwap => contract_env::revert(InvariantErrorReturn::NoGainSwap),
            InvariantError::InvalidTickSpacing => {
                contract_env::revert(InvariantErrorReturn::InvalidTickSpacing)
            }
            InvariantError::FeeTierAlreadyExist => {
                contract_env::revert(InvariantErrorReturn::FeeTierAlreadyExist)
            }
            InvariantError::PoolKeyAlreadyExist => {
                contract_env::revert(InvariantErrorReturn::PoolKeyAlreadyExist)
            }
            InvariantError::UnauthorizedFeeReceiver => {
                contract_env::revert(InvariantErrorReturn::UnauthorizedFeeReceiver)
            }
            InvariantError::ZeroLiquidity => {
                contract_env::revert(InvariantErrorReturn::ZeroLiquidity)
            }
            InvariantError::TransferError => {
                contract_env::revert(InvariantErrorReturn::TransferError)
            }
            InvariantError::TokensAreSame => {
                contract_env::revert(InvariantErrorReturn::TokensAreSame)
            }
            InvariantError::AmountUnderMinimumAmountOut => {
                contract_env::revert(InvariantErrorReturn::AmountUnderMinimumAmountOut)
            }
            InvariantError::InvalidFee => contract_env::revert(InvariantErrorReturn::InvalidFee),
            InvariantError::NotEmptyTickDeinitialization => {
                contract_env::revert(InvariantErrorReturn::NotEmptyTickDeinitialization)
            }
            InvariantError::InvalidInitTick => {
                contract_env::revert(InvariantErrorReturn::InvalidInitTick)
            }
            InvariantError::InvalidInitSqrtPrice => {
                contract_env::revert(InvariantErrorReturn::InvalidInitSqrtPrice)
            }
        },
    }
}
