use crate::route::router;
use poem::{listener::TcpListener, Server};

#[tokio::main]
pub async fn server() -> Result<(), std::io::Error> {
    let app = router::generate();
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
