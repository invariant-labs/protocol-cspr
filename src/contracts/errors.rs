use odra::execution_error;

execution_error! {
    pub enum InvariantExecutionError {
        TickAlreadyExist => 1,
        TickNotFound => 2,
        PoolAlreadyExist => 3,
        PoolNotFound => 4,
    }
}
