use crate::auth::AuthService;
use crate::error::SdkError;
use crate::permission::PermissionService;
use fyc_db::DbPool;
use fyc_db::repositories::{AuditRepo, OrderRepo, ProductRepo};

pub struct OrderService {
    pool: DbPool,
    product_repo: ProductRepo,
    order_repo: OrderRepo,
    audit_repo: AuditRepo,
    auth: AuthService,
    permission: PermissionService,
}

impl OrderService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            product_repo: ProductRepo::new(pool.clone()),
            order_repo: OrderRepo::new(pool.clone()),
            audit_repo: AuditRepo::new(pool.clone()),
            auth: AuthService::new(pool.clone()),
            permission: PermissionService::new(pool.clone()),
            pool,
        }
    }

    pub fn create_order(&self, token: &str, items: &[(i64, i32)]) -> Result<i64, SdkError> {
        let user_id = self.auth.validate_token(token)?;
        if !self.permission.has_permission(user_id, "order:create")? {
            return Err(SdkError::AuthFailed(
                "Missing permission: order:create".into(),
            ));
        }

        let conn = self.pool.get().map_err(fyc_db::DbError::from)?;
        conn.execute("BEGIN", []).map_err(fyc_db::DbError::from)?;

        let mut total = 0.0;
        let mut order_items = Vec::new();

        for &(product_id, quantity) in items {
            let product = self
                .product_repo
                .find_by_id_with_conn(&conn, product_id)?
                .ok_or_else(|| SdkError::NotFound(format!("Product {} not found", product_id)))?;
            let item_total = product.price * quantity as f64;
            total += item_total;
            order_items.push((product_id, quantity, product.price));
        }

        let order_id = self
            .order_repo
            .create_order_with_conn(&conn, user_id, "paid", total)?;
        for (product_id, quantity, unit_price) in order_items {
            OrderRepo::add_order_item_with_conn(&conn, order_id, product_id, quantity, unit_price)?;
        }

        conn.execute("COMMIT", []).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            fyc_db::DbError::from(e)
        })?;

        self.audit_repo.log(
            user_id,
            "order:create",
            Some(order_id),
            Some(&format!("total: {}", total)),
        )?;
        Ok(order_id)
    }

    pub fn get_today_orders(&self, token: &str) -> Result<Vec<fyc_db::Order>, SdkError> {
        let user_id = self.auth.validate_token(token)?;
        if !self.permission.has_permission(user_id, "order:view")? {
            return Err(SdkError::AuthFailed(
                "Missing permission: order:view".into(),
            ));
        }
        Ok(self.order_repo.get_orders_today()?)
    }

    pub fn get_order_detail(
        &self,
        token: &str,
        order_id: i64,
    ) -> Result<(fyc_db::Order, Vec<fyc_db::OrderItem>), SdkError> {
        let user_id = self.auth.validate_token(token)?;
        if !self.permission.has_permission(user_id, "order:view")? {
            return Err(SdkError::AuthFailed(
                "Missing permission: order:view".into(),
            ));
        }
        let order = self
            .order_repo
            .find_order_by_id(order_id)?
            .ok_or_else(|| SdkError::NotFound("Order not found".into()))?;
        let items = self.order_repo.get_order_items(order_id)?;
        Ok((order, items))
    }
}
