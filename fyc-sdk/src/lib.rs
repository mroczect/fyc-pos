pub mod auth;
pub mod crypto;
pub mod error;
pub mod menu;
pub mod order;
pub mod permission;
pub mod report;

pub use auth::AuthService;
pub use error::SdkError;
pub use menu::MenuService;
pub use order::OrderService;
pub use report::ReportService;

use fyc_db::repositories::{PermissionRepo, ProductCustomRepo, RoleRepo};
use fyc_db::{DbError, DbPool};

pub fn seed_defaults(pool: &DbPool) -> Result<(), SdkError> {
    let role_repo = RoleRepo::new(pool.clone());
    let perm_repo = PermissionRepo::new(pool.clone());
    let custom_repo = ProductCustomRepo::new(pool.clone());

    if role_repo.get_role_by_name("admin")?.is_none()
        && let Err(DbError::DuplicateEntry(_)) = role_repo.create_role("admin", "Administrator")
    {
    }
    if role_repo.get_role_by_name("kasir")?.is_none()
        && let Err(DbError::DuplicateEntry(_)) = role_repo.create_role("kasir", "Cashier")
    {}

    let perms = vec![
        ("user:create", "Create new users"),
        ("user:deactivate", "Deactivate users"),
        ("role:assign", "Assign roles to users"),
        ("role:remove", "Remove roles from users"),
        ("product:create", "Create products"),
        ("product:update", "Update products"),
        ("product:delete", "Delete products"),
        ("customfield:manage", "Manage custom fields"),
        ("order:create", "Create orders"),
        ("order:view", "View orders"),
        ("report:view", "View reports"),
    ];

    for (name, desc) in &perms {
        if perm_repo.get_by_name(name)?.is_none()
            && let Err(DbError::DuplicateEntry(_)) = perm_repo.create(name, desc)
        {}
    }

    if let Some(admin_role) = role_repo.get_role_by_name("admin")? {
        for (name, _) in &perms {
            if let Some(perm) = perm_repo.get_by_name(name)?
                && let Err(DbError::DuplicateEntry(_)) =
                    role_repo.assign_permission_to_role(admin_role.id, perm.id)
            {}
        }
    }

    if custom_repo.get_field_by_name("varian")?.is_none()
        && let Err(DbError::DuplicateEntry(_)) = custom_repo.create_field("varian", "text")
    {}

    Ok(())
}
