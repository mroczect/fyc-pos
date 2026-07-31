pub mod connection;
pub mod error;
pub mod models;
pub mod repositories;
pub mod validation;

pub use error::DbError;
pub use models::*;
pub use repositories::sqlite;
pub use repositories::traits;
pub use validation::*;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
pub type DbPool = Pool<SqliteConnectionManager>;
