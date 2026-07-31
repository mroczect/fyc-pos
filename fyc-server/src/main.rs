#[actix_web::main]
async fn main() -> std::io::Result<()> {
    fyc_server::run_server().await
}
