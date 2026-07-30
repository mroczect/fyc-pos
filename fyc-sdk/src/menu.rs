use crate::auth::AuthService;
use crate::error::SdkError;
use crate::permission::PermissionService;
use fyc_db::repositories::{AuditRepo, ProductCustomRepo, ProductRepo};
use fyc_db::{DbPool, Product};

pub struct MenuService {
    product_repo: ProductRepo,
    custom_repo: ProductCustomRepo,
    audit_repo: AuditRepo,
    auth: AuthService,
    permission: PermissionService,
}

impl MenuService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            product_repo: ProductRepo::new(pool.clone()),
            custom_repo: ProductCustomRepo::new(pool.clone()),
            audit_repo: AuditRepo::new(pool.clone()),
            auth: AuthService::new(pool.clone()),
            permission: PermissionService::new(pool.clone()),
        }
    }

    pub fn add_product(
        &self,
        token: &str,
        name: &str,
        price: f64,
        category: &str,
    ) -> Result<i64, SdkError> {
        let user_id = self.auth.validate_token(token)?;
        if !self.permission.has_permission(user_id, "product:create")? {
            return Err(SdkError::AuthFailed(
                "Missing permission: product:create".into(),
            ));
        }
        let id = self.product_repo.create(name, price, category)?;
        self.audit_repo.log(
            user_id,
            "product:create",
            Some(id),
            Some(&format!("{} - {}", name, price)),
        )?;
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
        let user_id = self.auth.validate_token(token)?;
        if !self.permission.has_permission(user_id, "product:update")? {
            return Err(SdkError::AuthFailed(
                "Missing permission: product:update".into(),
            ));
        }
        self.product_repo.update(id, name, price, category)?;
        self.audit_repo.log(
            user_id,
            "product:update",
            Some(id),
            Some(&format!("{} - {}", name, price)),
        )?;
        Ok(())
    }

    pub fn delete_product(&self, token: &str, id: i64) -> Result<(), SdkError> {
        let user_id = self.auth.validate_token(token)?;
        if !self.permission.has_permission(user_id, "product:delete")? {
            return Err(SdkError::AuthFailed(
                "Missing permission: product:delete".into(),
            ));
        }
        self.product_repo.deactivate(id)?;
        self.audit_repo
            .log(user_id, "product:delete", Some(id), None)?;
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
        let user_id = self.auth.validate_token(token)?;
        if !self
            .permission
            .has_permission(user_id, "customfield:manage")?
        {
            return Err(SdkError::AuthFailed(
                "Missing permission: customfield:manage".into(),
            ));
        }
        let id = self.custom_repo.create_field(name, field_type)?;
        self.audit_repo
            .log(user_id, "customfield:create", Some(id), Some(name))?;
        Ok(id)
    }

    pub fn set_custom_value(
        &self,
        token: &str,
        product_id: i64,
        field_name: &str,
        value: &str,
    ) -> Result<(), SdkError> {
        let user_id = self.auth.validate_token(token)?;
        if !self
            .permission
            .has_permission(user_id, "customfield:manage")?
        {
            return Err(SdkError::AuthFailed(
                "Missing permission: customfield:manage".into(),
            ));
        }
        let field = self
            .custom_repo
            .get_field_by_name(field_name)?
            .ok_or_else(|| SdkError::NotFound("Custom field not found".into()))?;
        self.custom_repo.set_value(product_id, field.id, value)?;
        self.audit_repo.log(
            user_id,
            "customfield:set_value",
            Some(product_id),
            Some(&format!("{}={}", field_name, value)),
        )?;
        Ok(())
    }

    pub fn get_product_custom_values(
        &self,
        token: &str,
        product_id: i64,
    ) -> Result<Vec<(fyc_db::ProductCustomField, String)>, SdkError> {
        let _ = self.auth.validate_token(token)?;
        Ok(self.custom_repo.get_values_for_product(product_id)?)
    }
}
