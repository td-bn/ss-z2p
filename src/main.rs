use std::net::TcpListener;
use z2p::run;

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    run(TcpListener::bind("127.0.0.1:8000").unwrap())?.await
}
