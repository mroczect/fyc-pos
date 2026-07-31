use fyc_db::DbPool;
use fyc_sdk::{AuthService, MenuService, OrderService, ReportService};

pub struct AppState {
    pool: DbPool,
}

impl AppState {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn auth_service(&self) -> AuthService {
        AuthService::new(self.pool.clone())
    }

    pub fn menu_service(&self) -> MenuService {
        MenuService::new(self.pool.clone())
    }

    pub fn order_service(&self) -> OrderService {
        OrderService::new(self.pool.clone())
    }

    pub fn report_service(&self) -> ReportService {
        ReportService::new(self.pool.clone())
    }
}
