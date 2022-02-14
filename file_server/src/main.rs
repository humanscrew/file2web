pub mod service {
    pub mod router;
}

pub mod api {
    pub mod file;
    pub mod file_dir;
    pub mod greet;
}

use crate::service::router;
use poem::{listener::TcpListener, Server};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = router::generate();
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
