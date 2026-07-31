use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database pool creation failed: {0}")]
    PoolCreation(String),

    #[error("Database migration failed: {0}")]
    MigrationFailed(String),

    #[error("Database query failed: {0}")]
    QueryError(#[from] rusqlite::Error),

    #[error("Database pool error: {0}")]
    PoolError(#[from] r2d2::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    #[error("Foreign key violation: {0}")]
    ForeignKeyViolation(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
