use crate::DbPool;
use crate::error::DbError;
use crate::models::{Order, OrderItem};
use crate::repositories::traits::OrderRepository;
use crate::validation;
use rusqlite::{ErrorCode, params};

pub struct OrderRepo {
    pool: DbPool,
}

impl OrderRepo {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create_order_with_conn(
        conn: &rusqlite::Connection,
        user_id: i64,
        status: &str,
        total: f64,
    ) -> Result<i64, DbError> {
        Self::validate_status(status)?;
        Self::insert_order(conn, user_id, status, total)
    }

    pub fn add_order_item_with_conn(
        conn: &rusqlite::Connection,
        order_id: i64,
        product_id: i64,
        quantity: i32,
        unit_price: f64,
    ) -> Result<i64, DbError> {
        validation::validate_quantity(quantity)?;
        Self::insert_order_item(conn, order_id, product_id, quantity, unit_price)
    }

    fn validate_status(status: &str) -> Result<(), DbError> {
        if !["pending", "paid", "cancelled"].contains(&status) {
            return Err(DbError::InvalidInput("Invalid order status".into()));
        }
        Ok(())
    }

    pub fn insert_order(
        conn: &rusqlite::Connection,
        user_id: i64,
        status: &str,
        total: f64,
    ) -> Result<i64, DbError> {
        match conn.execute(
            "INSERT INTO orders (user_id, status, total) VALUES (?1, ?2, ?3)",
            params![user_id, status, total],
        ) {
            Ok(_) => Ok(conn.last_insert_rowid()),
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == ErrorCode::ConstraintViolation =>
            {
                Err(DbError::ForeignKeyViolation("User does not exist".into()))
            }
            Err(e) => Err(DbError::QueryError(e)),
        }
    }

    pub fn insert_order_item(
        conn: &rusqlite::Connection,
        order_id: i64,
        product_id: i64,
        quantity: i32,
        unit_price: f64,
    ) -> Result<i64, DbError> {
        match conn.execute(
            "INSERT INTO order_items (order_id, product_id, quantity, unit_price) VALUES (?1, ?2, ?3, ?4)",
            params![order_id, product_id, quantity, unit_price],
        ) {
            Ok(_) => Ok(conn.last_insert_rowid()),
            Err(rusqlite::Error::SqliteFailure(err, _)) if err.code == ErrorCode::ConstraintViolation =>
                Err(DbError::ForeignKeyViolation("Order or Product does not exist".into())),
            Err(e) => Err(DbError::QueryError(e)),
        }
    }

    pub fn create_order(&self, user_id: i64, status: &str, total: f64) -> Result<i64, DbError> {
        <Self as OrderRepository>::create_order(self, user_id, status, total)
    }

    pub fn add_order_item(
        &self,
        order_id: i64,
        product_id: i64,
        quantity: i32,
        unit_price: f64,
    ) -> Result<i64, DbError> {
        <Self as OrderRepository>::add_order_item(self, order_id, product_id, quantity, unit_price)
    }

    pub fn get_orders_today(&self) -> Result<Vec<Order>, DbError> {
        <Self as OrderRepository>::get_orders_today(self)
    }

    pub fn get_order_items(&self, order_id: i64) -> Result<Vec<OrderItem>, DbError> {
        <Self as OrderRepository>::get_order_items(self, order_id)
    }

    pub fn find_order_by_id(&self, order_id: i64) -> Result<Option<Order>, DbError> {
        <Self as OrderRepository>::find_order_by_id(self, order_id)
    }

    pub fn query_orders_today(conn: &rusqlite::Connection) -> Result<Vec<Order>, DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, user_id, status, total, created_at FROM orders WHERE date(created_at) = date('now') ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Order {
                id: row.get(0)?,
                user_id: row.get(1)?,
                status: row.get(2)?,
                total: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        let mut orders = Vec::new();
        for o in rows {
            orders.push(o?);
        }
        Ok(orders)
    }

    pub fn query_order_items(
        conn: &rusqlite::Connection,
        order_id: i64,
    ) -> Result<Vec<OrderItem>, DbError> {
        let mut stmt = conn.prepare("SELECT id, order_id, product_id, quantity, unit_price FROM order_items WHERE order_id = ?1")?;
        let rows = stmt.query_map(params![order_id], |row| {
            Ok(OrderItem {
                id: row.get(0)?,
                order_id: row.get(1)?,
                product_id: row.get(2)?,
                quantity: row.get(3)?,
                unit_price: row.get(4)?,
            })
        })?;
        let mut items = Vec::new();
        for i in rows {
            items.push(i?);
        }
        Ok(items)
    }

    pub fn query_order_by_id(
        conn: &rusqlite::Connection,
        order_id: i64,
    ) -> Result<Option<Order>, DbError> {
        let mut stmt = conn
            .prepare("SELECT id, user_id, status, total, created_at FROM orders WHERE id = ?1")?;
        let mut rows = stmt.query_map(params![order_id], |row| {
            Ok(Order {
                id: row.get(0)?,
                user_id: row.get(1)?,
                status: row.get(2)?,
                total: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        match rows.next() {
            Some(order) => Ok(Some(order?)),
            None => Ok(None),
        }
    }
}

impl OrderRepository for OrderRepo {
    fn create_order(&self, user_id: i64, status: &str, total: f64) -> Result<i64, DbError> {
        Self::validate_status(status)?;
        let conn = self.pool.get()?;
        Self::insert_order(&conn, user_id, status, total)
    }

    fn add_order_item(
        &self,
        order_id: i64,
        product_id: i64,
        quantity: i32,
        unit_price: f64,
    ) -> Result<i64, DbError> {
        validation::validate_quantity(quantity)?;
        let conn = self.pool.get()?;
        Self::insert_order_item(&conn, order_id, product_id, quantity, unit_price)
    }

    fn get_orders_today(&self) -> Result<Vec<Order>, DbError> {
        let conn = self.pool.get()?;
        Self::query_orders_today(&conn)
    }

    fn get_order_items(&self, order_id: i64) -> Result<Vec<OrderItem>, DbError> {
        let conn = self.pool.get()?;
        Self::query_order_items(&conn, order_id)
    }

    fn find_order_by_id(&self, order_id: i64) -> Result<Option<Order>, DbError> {
        let conn = self.pool.get()?;
        Self::query_order_by_id(&conn, order_id)
    }
}
