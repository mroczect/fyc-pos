use thiserror::Error;

#[derive(Error, Debug)]
pub enum RoleError {
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}
