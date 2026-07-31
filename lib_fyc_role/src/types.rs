use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Developer,
    Admin,
    User,
}

impl Role {
    pub fn can(&self, permission: Permission) -> bool {
        match self {
            Role::Developer => true,
            Role::Admin => !matches!(
                permission,
                Permission::ChangePassword | Permission::ManageUsers
            ),
            Role::User => matches!(
                permission,
                Permission::CreateOrder | Permission::ReadOrder | Permission::UpdateOrderStatus
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    CreateOrder,
    ReadOrder,
    UpdateOrderStatus,
    ManageProducts,
    ViewSalesReport,
    ChangePassword,
    ManageUsers,
}
