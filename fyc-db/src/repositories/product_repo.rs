use crate::error::DbError;
use crate::models::Product;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{ErrorCode, params};

pub struct ProductRepo {
    pool: Pool<SqliteConnectionManager>,
}

impl ProductRepo {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn create(&self, name: &str, price: f64, category: &str) -> Result<i64, DbError> {
        let conn = self.pool.get()?;
        Self::insert(&conn, name, price, category)
    }

    pub fn create_with_conn(
        conn: &rusqlite::Connection,
        name: &str,
        price: f64,
        category: &str,
    ) -> Result<i64, DbError> {
        Self::insert(conn, name, price, category)
    }

    fn insert(
        conn: &rusqlite::Connection,
        name: &str,
        price: f64,
        category: &str,
    ) -> Result<i64, DbError> {
        if name.trim().is_empty() {
            return Err(DbError::InvalidInput("Name cannot be empty".into()));
        }
        if price < 0.0 {
            return Err(DbError::InvalidInput("Price cannot be negative".into()));
        }
        match conn.execute(
            "INSERT INTO products (name, price, category) VALUES (?1, ?2, ?3)",
            params![name.trim(), price, category.trim()],
        ) {
            Ok(_) => Ok(conn.last_insert_rowid()),
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == ErrorCode::ConstraintViolation =>
            {
                Err(DbError::DuplicateEntry("Product already exists".into()))
            }
            Err(e) => Err(DbError::QueryError(e)),
        }
    }

    pub fn find_by_id(&self, id: i64) -> Result<Option<Product>, DbError> {
        let conn = self.pool.get()?;
        Self::query_by_id(&conn, id)
    }

    pub fn find_by_id_with_conn(
        conn: &rusqlite::Connection,
        id: i64,
    ) -> Result<Option<Product>, DbError> {
        Self::query_by_id(conn, id)
    }

    fn query_by_id(conn: &rusqlite::Connection, id: i64) -> Result<Option<Product>, DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, name, price, category, is_active, created_at, updated_at
             FROM products WHERE id = ?1 AND is_active = 1",
        )?;
        let mut rows = stmt.query_map(params![id], Self::row_to_product)?;
        match rows.next() {
            Some(p) => Ok(Some(p?)),
            None => Ok(None),
        }
    }

    pub fn find_all_active(&self) -> Result<Vec<Product>, DbError> {
        let conn = self.pool.get()?;
        Self::query_all_active(&conn)
    }

    fn query_all_active(conn: &rusqlite::Connection) -> Result<Vec<Product>, DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, name, price, category, is_active, created_at, updated_at
             FROM products WHERE is_active = 1 ORDER BY name",
        )?;
        let rows = stmt.query_map([], Self::row_to_product)?;
        let mut products = Vec::new();
        for p in rows {
            products.push(p?);
        }
        Ok(products)
    }

    pub fn update(&self, id: i64, name: &str, price: f64, category: &str) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        Self::update_with_conn(&conn, id, name, price, category)
    }

    pub fn update_with_conn(
        conn: &rusqlite::Connection,
        id: i64,
        name: &str,
        price: f64,
        category: &str,
    ) -> Result<(), DbError> {
        if name.trim().is_empty() {
            return Err(DbError::InvalidInput("Name cannot be empty".into()));
        }
        if price < 0.0 {
            return Err(DbError::InvalidInput("Price cannot be negative".into()));
        }
        let affected = conn.execute(
            "UPDATE products SET name = ?1, price = ?2, category = ?3, updated_at = datetime('now')
             WHERE id = ?4 AND is_active = 1",
            params![name.trim(), price, category.trim(), id],
        )?;
        if affected == 0 {
            Err(DbError::NotFound("Product not found".into()))
        } else {
            Ok(())
        }
    }

    pub fn deactivate(&self, id: i64) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        Self::deactivate_with_conn(&conn, id)
    }

    pub fn deactivate_with_conn(conn: &rusqlite::Connection, id: i64) -> Result<(), DbError> {
        let affected = conn.execute(
            "UPDATE products SET is_active = 0, updated_at = datetime('now') WHERE id = ?1",
            params![id],
        )?;
        if affected == 0 {
            Err(DbError::NotFound("Product not found".into()))
        } else {
            Ok(())
        }
    }

    fn row_to_product(row: &rusqlite::Row) -> rusqlite::Result<Product> {
        Ok(Product {
            id: row.get(0)?,
            name: row.get(1)?,
            price: row.get(2)?,
            category: row.get(3)?,
            is_active: row.get::<_, i32>(4)? == 1,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }
}
