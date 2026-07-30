use crate::auth::AuthService;
use crate::error::SdkError;
use crate::permission::PermissionService;
use fyc_db::repositories::{AuditRepo, OrderRepo, ProductRepo};
use fyc_db::{DbError, DbPool, Order, OrderItem};

pub struct OrderService {
    pool: DbPool,
    order_repo: OrderRepo,
    auth: AuthService,
    permission: PermissionService,
}

impl OrderService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            order_repo: OrderRepo::new(pool.clone()),
            auth: AuthService::new(pool.clone()),
            permission: PermissionService::new(pool.clone()),
            pool,
        }
    }

    fn check_permission(&self, token: &str, permission: &str) -> Result<i64, SdkError> {
        let user_id = self.auth.validate_token(token)?;
        if !self.permission.has_permission(user_id, permission)? {
            return Err(SdkError::AuthFailed(format!(
                "Missing permission: {}",
                permission
            )));
        }
        Ok(user_id)
    }

    pub fn create_order(&self, token: &str, items: &[(i64, i32)]) -> Result<i64, SdkError> {
        let user_id = self.check_permission(token, "order:create")?;

        let conn = self.pool.get().map_err(DbError::from)?;
        conn.execute("BEGIN", []).map_err(DbError::from)?;

        let mut total = 0.0;
        let mut order_items = Vec::new();

        for &(product_id, quantity) in items {
            let product = ProductRepo::find_by_id_with_conn(&conn, product_id)
                .inspect_err(|_| {
                    let _ = conn.execute("ROLLBACK", []);
                })?
                .ok_or_else(|| {
                    let _ = conn.execute("ROLLBACK", []);
                    SdkError::NotFound(format!("Product {} not found", product_id))
                })?;

            let item_total = product.price * quantity as f64;
            total += item_total;
            order_items.push((product_id, quantity, product.price));
        }

        let order_id = self
            .order_repo
            .create_order_with_conn(&conn, user_id, "paid", total)
            .inspect_err(|_| {
                let _ = conn.execute("ROLLBACK", []);
            })?;

        for &(product_id, quantity, unit_price) in &order_items {
            OrderRepo::add_order_item_with_conn(&conn, order_id, product_id, quantity, unit_price)
                .inspect_err(|_| {
                    let _ = conn.execute("ROLLBACK", []);
                })?;
        }

        AuditRepo::log_with_conn(
            &conn,
            user_id,
            "order:create",
            Some(order_id),
            Some(&format!("total: {}", total)),
        )
        .inspect_err(|_| {
            let _ = conn.execute("ROLLBACK", []);
        })?;

        conn.execute("COMMIT", [])
            .inspect_err(|_| {
                let _ = conn.execute("ROLLBACK", []);
            })
            .map_err(DbError::from)?;

        Ok(order_id)
    }

    pub fn get_today_orders(&self, token: &str) -> Result<Vec<Order>, SdkError> {
        let _ = self.check_permission(token, "order:view")?;
        Ok(self.order_repo.get_orders_today()?)
    }

    pub fn get_order_detail(
        &self,
        token: &str,
        order_id: i64,
    ) -> Result<(Order, Vec<OrderItem>), SdkError> {
        let _ = self.check_permission(token, "order:view")?;
        let order = self
            .order_repo
            .find_order_by_id(order_id)?
            .ok_or_else(|| SdkError::NotFound("Order not found".into()))?;
        let items = self.order_repo.get_order_items(order_id)?;
        Ok((order, items))
    }
}
