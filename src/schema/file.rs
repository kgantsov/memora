use charybdis::types::{Text, Timestamp};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::model::file::File;

#[derive(Deserialize, Debug, Validate)]
pub struct FileCreateRequest {
    pub name: Text,
    pub directory: Text,
    pub status: Text,
}

#[derive(Deserialize, Debug, Validate)]
pub struct FileUpdateRequest {
    pub name: Text,
    pub directory: Text,
    pub status: Text,
    pub created_at: Timestamp,
    pub modified_at: Timestamp,
}

#[derive(Serialize)]
pub struct FilesResponse {
    pub objects: Vec<File>,
}
