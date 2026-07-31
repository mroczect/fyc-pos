pub mod app_state;
pub mod routes;

use actix_web::{App, HttpServer, middleware, web};
use app_state::AppState;
use fyc_db::connection::create_pool;

pub async fn run_server() -> std::io::Result<()> {
    let pool = create_pool("fyc.db").expect("Failed to create pool");
    fyc_sdk::seed_defaults(&pool).expect("Failed to seed defaults");

    let state = web::Data::new(AppState::new(pool));

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);

    let mut current_port = port;
    let server = loop {
        let addr = format!("127.0.0.1:{}", current_port);
        match HttpServer::new({
            let state = state.clone();
            move || {
                App::new()
                    .app_data(state.clone())
                    .wrap(middleware::Logger::default())
                    .configure(routes::configure)
            }
        })
        .bind(&addr)
        {
            Ok(srv) => {
                println!("Server running at http://{addr}");
                break srv;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                current_port += 1;
                if current_port > port + 20 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::AddrInUse,
                        format!("No available ports in range {}-{}", port, port + 20),
                    ));
                }
            }
            Err(e) => return Err(e),
        }
    };

    server.run().await
}
