use crate::error::DbError;
use crate::models::ProductCustomField;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

pub struct ProductCustomRepo {
    pool: Pool<SqliteConnectionManager>,
}

impl ProductCustomRepo {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn create_field(&self, name: &str, field_type: &str) -> Result<i64, DbError> {
        let conn = self.pool.get()?;
        Self::insert_field(&conn, name, field_type)
    }
    pub fn create_field_with_conn(
        &self,
        conn: &rusqlite::Connection,
        name: &str,
        field_type: &str,
    ) -> Result<i64, DbError> {
        Self::insert_field(conn, name, field_type)
    }
    fn insert_field(
        conn: &rusqlite::Connection,
        name: &str,
        field_type: &str,
    ) -> Result<i64, DbError> {
        if name.trim().is_empty() {
            return Err(DbError::InvalidInput("Field name cannot be empty".into()));
        }
        if !["text", "number", "boolean"].contains(&field_type) {
            return Err(DbError::InvalidInput("Invalid field type".into()));
        }
        match conn.execute(
            "INSERT INTO product_custom_fields (name, field_type) VALUES (?1, ?2)",
            params![name.trim(), field_type],
        ) {
            Ok(_) => Ok(conn.last_insert_rowid()),
            Err(e) => Err(DbError::QueryError(e)),
        }
    }

    pub fn get_field_by_name(&self, name: &str) -> Result<Option<ProductCustomField>, DbError> {
        let conn = self.pool.get()?;
        Self::query_field_by_name(&conn, name)
    }
    pub fn get_field_by_name_with_conn(
        conn: &rusqlite::Connection,
        name: &str,
    ) -> Result<Option<ProductCustomField>, DbError> {
        Self::query_field_by_name(conn, name)
    }
    fn query_field_by_name(
        conn: &rusqlite::Connection,
        name: &str,
    ) -> Result<Option<ProductCustomField>, DbError> {
        let mut stmt =
            conn.prepare("SELECT id, name, field_type FROM product_custom_fields WHERE name = ?1")?;
        let mut rows = stmt.query_map(params![name], |row| {
            Ok(ProductCustomField {
                id: row.get(0)?,
                name: row.get(1)?,
                field_type: row.get(2)?,
            })
        })?;
        match rows.next() {
            Some(f) => Ok(Some(f?)),
            None => Ok(None),
        }
    }

    pub fn set_value(&self, product_id: i64, field_id: i64, value: &str) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        Self::set_value_with_conn(&conn, product_id, field_id, value)
    }
    pub fn set_value_with_conn(
        conn: &rusqlite::Connection,
        product_id: i64,
        field_id: i64,
        value: &str,
    ) -> Result<(), DbError> {
        conn.execute(
            "INSERT OR REPLACE INTO product_custom_values (product_id, field_id, value) VALUES (?1, ?2, ?3)",
            params![product_id, field_id, value],
        )?;
        Ok(())
    }

    pub fn get_values_for_product(
        &self,
        product_id: i64,
    ) -> Result<Vec<(ProductCustomField, String)>, DbError> {
        let conn = self.pool.get()?;
        Self::query_values_for_product(&conn, product_id)
    }
    fn query_values_for_product(
        conn: &rusqlite::Connection,
        product_id: i64,
    ) -> Result<Vec<(ProductCustomField, String)>, DbError> {
        let mut stmt = conn.prepare(
            "SELECT f.id, f.name, f.field_type, v.value FROM product_custom_fields f JOIN product_custom_values v ON f.id = v.field_id WHERE v.product_id = ?1"
        )?;
        let rows = stmt.query_map(params![product_id], |row| {
            Ok((
                ProductCustomField {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    field_type: row.get(2)?,
                },
                row.get::<_, String>(3)?,
            ))
        })?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }
}
