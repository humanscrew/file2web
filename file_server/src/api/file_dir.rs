use async_recursion::async_recursion;
use futures::future;
use poem::{
    error::StaticFileError,
    handler,
    web::{Json, Query},
    Result,
};
use serde::{Deserialize, Serialize};
use std::{
    // fs::{FileType, Permissions},
    fs,
    io,
    path::PathBuf,
    // time::SystemTime,
};
use sysinfo::{DiskExt, System, SystemExt};

#[derive(Debug, Deserialize, Serialize)]
pub struct Dir<T> {
    pub path: Option<T>,
    pub metadata: Option<Metadata>,
    pub children: Option<Vec<Dir<T>>>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Metadata {
    // pub file_type: FileType,
    pub is_dir: bool,
    pub is_file: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub status: bool,
    pub message: Option<String>,
    // pub permissions: Permissions,
    // pub modified: Result<SystemTime>,
    // pub accessed: Result<SystemTime>,
    // pub created: Result<SystemTime>,
}

impl Metadata {
    fn new(metadata: &fs::Metadata) -> Self {
        let is_dir = metadata.is_dir();
        let size = if is_dir { 0 } else { metadata.len() };
        Self {
            is_dir,
            is_file: metadata.is_file(),
            is_symlink: metadata.is_symlink(),
            size,
            status: true,
            message: None,
        }
    }

    fn error(message: String) -> Self {
        Self {
            message: Some(message),
            ..Default::default()
        }
    }
}

impl Dir<String> {
    async fn new(dir_path: impl Into<PathBuf>, depth: usize) -> Result<Self, io::Error> {
        let dir_path_buf = dir_path.into();
        Self::map_dir(dir_path_buf, depth).await
    }

    #[async_recursion]
    async fn map_dir(path_buf: PathBuf, depth: usize) -> Result<Self, io::Error> {
        let mut dir = Self {
            path: None,
            metadata: None,
            children: None,
        };

        let current_path = path_buf.as_path();
        if let Some(path) = current_path.to_str() {
            dir.path = Some(path.into());
        }
        match &fs::metadata(&current_path) {
            Ok(metadata) => dir.metadata = Some(Metadata::new(metadata)),
            Err(_) => dir.metadata = Some(Metadata::error("File Metadata Error".into())),
        };

        if depth <= 0 {
            return Ok(dir);
        }

        let mut async_queue = vec![];
        if path_buf.is_dir() {
            let read_dir = path_buf.read_dir()?;
            for entry in read_dir {
                let children_path_buf = entry?.path();
                let child = Self::map_dir(children_path_buf, depth - 1);
                async_queue.push(child);
            }
            let children = future::try_join_all(async_queue).await?;

            dir.children = Some(children);
        }
        Ok(dir)
    }

    async fn disk_dir(depth: usize) -> Result<Self, io::Error> {
        let mut dir = Self {
            path: Some('/'.into()),
            metadata: None,
            children: None,
        };

        let mut system = System::new();
        system.refresh_disks_list();
        let disk_mount_points: Vec<_> = system
            .disks()
            .iter()
            .map(|disk| disk.mount_point())
            .collect();

        let mut async_queue = vec![];
        for disk_mount_point in disk_mount_points {
            let disk_path = disk_mount_point.to_str();
            if let Some(path) = disk_path {
                let child = Self::new(path, depth);
                async_queue.push(child);
            }
        }
        let children = future::try_join_all(async_queue).await?;
        dir.children = Some(children);
        Ok(dir)
    }
}

#[derive(Debug, Deserialize)]
pub struct FileDirParams {
    path: Option<String>,
    depth: Option<usize>,
}

#[handler]
pub async fn file_dir(
    Query(params): Query<FileDirParams>,
) -> Result<Json<Dir<String>>, StaticFileError> {
    let FileDirParams { path, depth } = params;

    let file_dir: Dir<String>;
    if path.is_none() {
        let depth = depth.unwrap_or(0);
        file_dir = Dir::disk_dir(depth).await?
    } else {
        let depth = depth.unwrap_or(1);
        file_dir = Dir::new(path.unwrap(), depth).await?
    }
    Ok(Json(file_dir))
}
