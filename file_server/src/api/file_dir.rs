use async_recursion::async_recursion;
use futures::future;
use poem::{
    error::StaticFileError,
    handler,
    web::{Json, Query},
    Result,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use sysinfo::{DiskExt, System, SystemExt};
use tokio::{fs, io};

#[derive(Debug, Deserialize, Serialize, Default)]
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
    pub size: Option<u64>,
    pub status: bool,
    pub message: Option<String>,
    // pub permissions: Permissions,
    // pub modified: Result<SystemTime>,
    // pub accessed: Result<SystemTime>,
    // pub created: Result<SystemTime>,
}

impl Metadata {
    fn new(metadata: &std::fs::Metadata) -> Self {
        Self {
            is_file: metadata.is_file(),
            is_symlink: metadata.is_symlink(),
            status: true,
            ..Self::default()
        }
    }

    fn error(message: String) -> Self {
        Self {
            message: Some(message),
            ..Self::default()
        }
    }
}

impl Dir<String> {
    async fn new(dir_path: impl Into<PathBuf>, depth: isize) -> Result<Self, io::Error> {
        let dir_path_buf = dir_path.into();
        Self::make_dir(dir_path_buf, depth).await
    }

    #[async_recursion]
    async fn make_dir(path_buf: PathBuf, depth: isize) -> Result<Self, io::Error> {
        let mut dir = Self::default();
        dir.path = path_buf.to_str().map(str::to_string);

        let metadata = fs::metadata(&path_buf).await;
        let is_dir;
        match &metadata {
            Ok(metadata) => {
                dir.metadata = Some(Metadata::new(metadata));
                is_dir = metadata.is_dir();
                if let Some(dir_metadata) = &mut dir.metadata {
                    dir_metadata.is_dir = is_dir;
                    let size = if is_dir { None } else { Some(metadata.len()) };
                    dir_metadata.size = size;
                }
            }
            Err(err) => {
                dir.metadata = Some(Metadata::error(format!("{}", err)));
                return Ok(dir);
            }
        };

        if is_dir && depth != 0 {
            let mut async_queue = vec![];
            match path_buf.read_dir() {
                Ok(read_dir) => {
                    let child_depth = if depth > 0 { depth - 1 } else { -1 };
                    for entry in read_dir {
                        let children_path_buf = entry?.path();
                        let child = Self::make_dir(children_path_buf, child_depth);
                        async_queue.push(child);
                    }
                    let children = future::try_join_all(async_queue).await?;

                    dir.children = Some(children);
                }
                Err(err) => dir.metadata = Some(Metadata::error(format!("{}", err))),
            }
        };

        Ok(dir)
    }

    async fn disk_dir(depth: isize) -> Result<Self, io::Error> {
        let mut dir = Self {
            path: Some('/'.into()),
            ..Self::default()
        };

        if depth != 0 {
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
                    let child = Self::new(path, depth - 1);
                    async_queue.push(child);
                }
            }
            let children = future::try_join_all(async_queue).await?;
            dir.children = Some(children);
        }

        Ok(dir)
    }
}

#[derive(Debug, Deserialize)]
pub struct FileDirParams {
    path: Option<String>,
    depth: Option<isize>,
}

#[handler]
pub async fn file_dir(
    Query(params): Query<FileDirParams>,
) -> Result<Json<Dir<String>>, StaticFileError> {
    let FileDirParams { path, depth } = params;

    use std::time::Instant;
    let now = Instant::now();

    let file_dir: Dir<String>;
    let depth = depth.unwrap_or(1);
    if path.is_none() {
        file_dir = Dir::disk_dir(depth).await?
    } else {
        file_dir = Dir::new(path.unwrap(), depth).await?
    }

    println!("执行时间: {}ms", now.elapsed().as_millis());

    Ok(Json(file_dir))
}
