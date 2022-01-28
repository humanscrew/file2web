pub mod service {
    pub mod file_server;
}
pub mod route {
    pub mod router;
}

pub mod api {
    pub mod file;
    pub mod file_dir;
    pub mod greet;
}

pub fn main() {
    service::file_server::server().unwrap();
    ()
}
