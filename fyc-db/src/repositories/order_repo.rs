use crate::error::DbError;
use crate::models::{Order, OrderItem};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

pub struct OrderRepo {
    pool: Pool<SqliteConnectionManager>,
}

impl OrderRepo {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn create_order(&self, user_id: i64, status: &str, total: f64) -> Result<i64, DbError> {
        let conn = self.pool.get()?;
        Self::insert_order(&conn, user_id, status, total)
    }
    pub fn create_order_with_conn(
        &self,
        conn: &rusqlite::Connection,
        user_id: i64,
        status: &str,
        total: f64,
    ) -> Result<i64, DbError> {
        Self::insert_order(conn, user_id, status, total)
    }
    fn insert_order(
        conn: &rusqlite::Connection,
        user_id: i64,
        status: &str,
        total: f64,
    ) -> Result<i64, DbError> {
        conn.execute(
            "INSERT INTO orders (user_id, status, total) VALUES (?1, ?2, ?3)",
            params![user_id, status, total],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn add_order_item(
        &self,
        order_id: i64,
        product_id: i64,
        quantity: i32,
        unit_price: f64,
    ) -> Result<i64, DbError> {
        let conn = self.pool.get()?;
        Self::insert_order_item(&conn, order_id, product_id, quantity, unit_price)
    }
    pub fn add_order_item_with_conn(
        conn: &rusqlite::Connection,
        order_id: i64,
        product_id: i64,
        quantity: i32,
        unit_price: f64,
    ) -> Result<i64, DbError> {
        Self::insert_order_item(conn, order_id, product_id, quantity, unit_price)
    }
    fn insert_order_item(
        conn: &rusqlite::Connection,
        order_id: i64,
        product_id: i64,
        quantity: i32,
        unit_price: f64,
    ) -> Result<i64, DbError> {
        conn.execute("INSERT INTO order_items (order_id, product_id, quantity, unit_price) VALUES (?1, ?2, ?3, ?4)", params![order_id, product_id, quantity, unit_price])?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_orders_today(&self) -> Result<Vec<Order>, DbError> {
        let conn = self.pool.get()?;
        Self::query_orders_today(&conn)
    }
    fn query_orders_today(conn: &rusqlite::Connection) -> Result<Vec<Order>, DbError> {
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

    pub fn get_order_items(&self, order_id: i64) -> Result<Vec<OrderItem>, DbError> {
        let conn = self.pool.get()?;
        Self::query_order_items(&conn, order_id)
    }
    fn query_order_items(
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

    pub fn find_order_by_id(&self, order_id: i64) -> Result<Option<Order>, DbError> {
        let conn = self.pool.get()?;
        Self::query_order_by_id(&conn, order_id)
    }

    pub fn find_order_by_id_with_conn(
        conn: &rusqlite::Connection,
        order_id: i64,
    ) -> Result<Option<Order>, DbError> {
        Self::query_order_by_id(conn, order_id)
    }

    fn query_order_by_id(
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
