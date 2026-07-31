use crate::error::DbError;

pub fn validate_username(username: &str) -> Result<(), DbError> {
    if username.len() < 3 || username.len() > 30 {
        return Err(DbError::InvalidInput(
            "Username must be 3-30 characters".into(),
        ));
    }
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(DbError::InvalidInput(
            "Username can only contain alphanumeric and underscore".into(),
        ));
    }
    Ok(())
}

pub fn validate_role_name(name: &str) -> Result<(), DbError> {
    if name.trim().is_empty() {
        return Err(DbError::InvalidInput("Role name cannot be empty".into()));
    }
    if name.len() < 2 || name.len() > 50 {
        return Err(DbError::InvalidInput(
            "Role name must be 2-50 characters".into(),
        ));
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == ' ' || c == '-')
    {
        return Err(DbError::InvalidInput(
            "Role name can only contain alphanumeric, underscore, space, or hyphen".into(),
        ));
    }
    Ok(())
}

pub fn validate_product_name(name: &str) -> Result<(), DbError> {
    if name.trim().is_empty() {
        return Err(DbError::InvalidInput("Product name cannot be empty".into()));
    }
    if name.len() > 200 {
        return Err(DbError::InvalidInput(
            "Product name too long (max 200)".into(),
        ));
    }
    Ok(())
}

pub fn validate_price(price: f64) -> Result<(), DbError> {
    if price < 0.0 {
        return Err(DbError::InvalidInput("Price cannot be negative".into()));
    }
    if price > 1_000_000_000.0 {
        return Err(DbError::InvalidInput("Price seems unrealistic".into()));
    }
    Ok(())
}

pub fn validate_quantity(qty: i32) -> Result<(), DbError> {
    if qty <= 0 || qty > 100_000 {
        return Err(DbError::InvalidInput(
            "Quantity must be between 1 and 100000".into(),
        ));
    }
    Ok(())
}

pub fn validate_non_empty_text(field: &str, name: &str, max_len: usize) -> Result<(), DbError> {
    let trimmed = field.trim();
    if trimmed.is_empty() {
        return Err(DbError::InvalidInput(format!("{} cannot be empty", name)));
    }
    if trimmed.len() > max_len {
        return Err(DbError::InvalidInput(format!(
            "{} must be at most {} characters",
            name, max_len
        )));
    }
    Ok(())
}
