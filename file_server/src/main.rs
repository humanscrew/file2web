mod api;
mod service;

use crate::service::router;
use poem::{listener::TcpListener, Server};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = router::generate();
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
