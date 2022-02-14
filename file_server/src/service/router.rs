use crate::api::{file, file_dir, greet};
use poem::{get, EndpointExt, Route};

pub fn generate() -> Route {
    Route::new()
        .at("/greet/:name", get(greet::greet))
        .nest("/file", get(file::index).around(file::file))
        .at("file_dir", get(file_dir::file_dir))
}
