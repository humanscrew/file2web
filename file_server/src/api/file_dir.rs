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

#[derive(Debug, Deserialize, Serialize)]
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
            is_dir: false,
            is_file: false,
            is_symlink: false,
            size: 0,
            status: false,
            message: Some(message),
        }
    }
}

impl Dir<String> {
    fn new(dir_path: impl Into<PathBuf>, depth: usize) -> Result<Self, io::Error> {
        let dir_path = dir_path.into();
        Self::map_dir(dir_path, depth)
    }

    fn map_dir(path: PathBuf, depth: usize) -> Result<Self, io::Error> {
        let current_path = path.to_str().map(str::to_string);

        let mut dir = Self {
            path: current_path.clone(),
            metadata: None,
            children: None,
        };

        if let Some(file_path) = &current_path {
            match &fs::metadata(file_path) {
                Ok(metadata) => dir.metadata = Some(Metadata::new(metadata)),
                Err(_) => dir.metadata = Some(Metadata::error("File Failed".to_string())),
            }
        };

        if depth <= 0 {
            return Ok(dir);
        }

        if path.is_dir() {
            dir.children = Some(vec![]);
            let read_dir = path.read_dir()?;
            for entry in read_dir {
                let entry = entry?;
                let children_path = entry.path();
                let child = Self::map_dir(children_path, depth - 1)?;
                dir.children.as_mut().unwrap().push(child);
            }
        }

        Ok(dir)
    }

    fn disk_dir(depth: usize) -> Result<Self, io::Error> {
        let mut dir = Self {
            path: Some('/'.into()),
            metadata: None,
            children: Some(vec![]),
        };

        let mut system = System::new();
        system.refresh_disks_list();
        let disks: Vec<_> = system
            .disks()
            .iter()
            .map(|disk| disk.mount_point())
            .collect();

        for disk in disks {
            let path = disk.to_str();
            if let Some(path) = path {
                let child = Self::new(path, depth)?;
                dir.children.as_mut().unwrap().push(child);
            }
        }
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
        file_dir = Dir::disk_dir(depth)?
    } else {
        let depth = depth.unwrap_or(1);
        file_dir = Dir::new(path.unwrap(), depth)?
    }

    Ok(Json(file_dir))
}
