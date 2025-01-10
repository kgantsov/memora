use std::fmt;

use charybdis::types::{Text, Timestamp, Uuid};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::model::file::File;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FileType {
    FILE,
    DIRECTORY,
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FileStatus {
    OPEN,
    CLOSED,
}

impl fmt::Display for FileStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Debug, Validate)]
pub struct FileResponse {
    pub id: Uuid,
    pub name: Text,
    pub directory: Text,
    pub file_type: Text,
    pub status: Text,
    pub presigned_url: Option<Text>,
    pub upload_presigned_url: Option<Text>,
    pub created_at: Timestamp,
    pub modified_at: Timestamp,
}

#[derive(Deserialize, Debug, Validate)]
pub struct FileCreateRequest {
    pub name: Text,
    pub directory: Text,
    pub file_type: FileType,
    pub status: FileStatus,
}

#[derive(Deserialize, Debug, Validate)]
pub struct FileUpdateRequest {
    pub name: Text,
    pub directory: Text,
    pub file_type: FileType,
    pub status: FileStatus,
    pub created_at: Timestamp,
    pub modified_at: Timestamp,
}

#[derive(Serialize)]
pub struct FilesResponse {
    pub objects: Vec<File>,
}
