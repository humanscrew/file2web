use poem::{
    handler,
    web::{Path, Query},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UserParams {
    username: String,
}

#[handler]
pub async fn greet(
    Path(path): Path<String>,
    Query(UserParams { username }): Query<UserParams>,
) -> String {
    format!("Hello: {},{}", path, username)
}
