use crate::auth::AuthService;
use crate::error::SdkError;
use crate::permission::PermissionService;
use fyc_db::repositories::{AuditRepo, ProductCustomRepo, ProductRepo};
use fyc_db::{DbError, DbPool, Product, ProductCustomField};

pub struct MenuService {
    pool: DbPool,
    product_repo: ProductRepo,
    custom_repo: ProductCustomRepo,
    auth: AuthService,
    permission: PermissionService,
}

impl MenuService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            product_repo: ProductRepo::new(pool.clone()),
            custom_repo: ProductCustomRepo::new(pool.clone()),
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

    pub fn add_product(
        &self,
        token: &str,
        name: &str,
        price: f64,
        category: &str,
    ) -> Result<i64, SdkError> {
        let user_id = self.check_permission(token, "product:create")?;

        let conn = self.pool.get().map_err(DbError::from)?;
        conn.execute("BEGIN", []).map_err(DbError::from)?;

        let id = match ProductRepo::create_with_conn(&conn, name, price, category) {
            Ok(id) => id,
            Err(e) => {
                let _ = conn.execute("ROLLBACK", []);
                return Err(e.into());
            }
        };

        if let Err(e) = AuditRepo::log_with_conn(
            &conn,
            user_id,
            "product:create",
            Some(id),
            Some(&format!("{} - {}", name, price)),
        ) {
            let _ = conn.execute("ROLLBACK", []);
            return Err(e.into());
        }

        conn.execute("COMMIT", [])
            .inspect_err(|_| {
                let _ = conn.execute("ROLLBACK", []);
            })
            .map_err(DbError::from)?;
        Ok(id)
    }

    pub fn update_product(
        &self,
        token: &str,
        id: i64,
        name: &str,
        price: f64,
        category: &str,
    ) -> Result<(), SdkError> {
        let user_id = self.check_permission(token, "product:update")?;

        let conn = self.pool.get().map_err(DbError::from)?;
        conn.execute("BEGIN", []).map_err(DbError::from)?;

        if let Err(e) = ProductRepo::update_with_conn(&conn, id, name, price, category) {
            let _ = conn.execute("ROLLBACK", []);
            return Err(e.into());
        }

        if let Err(e) = AuditRepo::log_with_conn(
            &conn,
            user_id,
            "product:update",
            Some(id),
            Some(&format!("{} - {}", name, price)),
        ) {
            let _ = conn.execute("ROLLBACK", []);
            return Err(e.into());
        }

        conn.execute("COMMIT", [])
            .inspect_err(|_| {
                let _ = conn.execute("ROLLBACK", []);
            })
            .map_err(DbError::from)?;
        Ok(())
    }

    pub fn delete_product(&self, token: &str, id: i64) -> Result<(), SdkError> {
        let user_id = self.check_permission(token, "product:delete")?;

        let conn = self.pool.get().map_err(DbError::from)?;
        conn.execute("BEGIN", []).map_err(DbError::from)?;

        if let Err(e) = ProductRepo::deactivate_with_conn(&conn, id) {
            let _ = conn.execute("ROLLBACK", []);
            return Err(e.into());
        }

        if let Err(e) = AuditRepo::log_with_conn(&conn, user_id, "product:delete", Some(id), None) {
            let _ = conn.execute("ROLLBACK", []);
            return Err(e.into());
        }

        conn.execute("COMMIT", [])
            .inspect_err(|_| {
                let _ = conn.execute("ROLLBACK", []);
            })
            .map_err(DbError::from)?;
        Ok(())
    }

    pub fn list_products(&self, token: &str) -> Result<Vec<Product>, SdkError> {
        let _ = self.auth.validate_token(token)?;
        Ok(self.product_repo.find_all_active()?)
    }

    pub fn add_custom_field(
        &self,
        token: &str,
        name: &str,
        field_type: &str,
    ) -> Result<i64, SdkError> {
        let user_id = self.check_permission(token, "customfield:manage")?;

        let conn = self.pool.get().map_err(DbError::from)?;
        conn.execute("BEGIN", []).map_err(DbError::from)?;

        let id = match self
            .custom_repo
            .create_field_with_conn(&conn, name, field_type)
        {
            Ok(id) => id,
            Err(e) => {
                let _ = conn.execute("ROLLBACK", []);
                return Err(e.into());
            }
        };

        if let Err(e) =
            AuditRepo::log_with_conn(&conn, user_id, "customfield:create", Some(id), Some(name))
        {
            let _ = conn.execute("ROLLBACK", []);
            return Err(e.into());
        }

        conn.execute("COMMIT", [])
            .inspect_err(|_| {
                let _ = conn.execute("ROLLBACK", []);
            })
            .map_err(DbError::from)?;
        Ok(id)
    }

    pub fn set_custom_value(
        &self,
        token: &str,
        product_id: i64,
        field_name: &str,
        value: &str,
    ) -> Result<(), SdkError> {
        let user_id = self.check_permission(token, "customfield:manage")?;

        let field = self
            .custom_repo
            .get_field_by_name(field_name)?
            .ok_or_else(|| SdkError::NotFound("Custom field not found".into()))?;

        let conn = self.pool.get().map_err(DbError::from)?;
        conn.execute("BEGIN", []).map_err(DbError::from)?;

        if let Err(e) = ProductCustomRepo::set_value_with_conn(&conn, product_id, field.id, value) {
            let _ = conn.execute("ROLLBACK", []);
            return Err(e.into());
        }

        if let Err(e) = AuditRepo::log_with_conn(
            &conn,
            user_id,
            "customfield:set_value",
            Some(product_id),
            Some(&format!("{}={}", field_name, value)),
        ) {
            let _ = conn.execute("ROLLBACK", []);
            return Err(e.into());
        }

        conn.execute("COMMIT", [])
            .inspect_err(|_| {
                let _ = conn.execute("ROLLBACK", []);
            })
            .map_err(DbError::from)?;
        Ok(())
    }

    pub fn get_product_custom_values(
        &self,
        token: &str,
        product_id: i64,
    ) -> Result<Vec<(ProductCustomField, String)>, SdkError> {
        let _ = self.auth.validate_token(token)?;
        Ok(self.custom_repo.get_values_for_product(product_id)?)
    }
}
