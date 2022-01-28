use poem::{endpoint::StaticFilesEndpoint, handler, Endpoint, Request, Response, Result};
use serde::Deserialize;

#[handler]
pub async fn index() {}

#[derive(Debug, Deserialize)]
pub struct FileParams {
    drive_path: String,
}

pub async fn file<E: Endpoint>(_: E, req: Request) -> Result<Response> {
    let params = req.params::<FileParams>();
    let drive_path = match params {
        Ok(FileParams { drive_path }) => drive_path,
        _ => format!("/"),
    };
    StaticFilesEndpoint::new(drive_path)
        .show_files_listing()
        .call(req)
        .await
}
