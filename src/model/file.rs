use charybdis::macros::charybdis_model;
use charybdis::types::{Text, Timestamp, Uuid};
use serde::{Deserialize, Serialize};

use crate::schema::file::FileCreateRequest;
use crate::utils::node::generate_uuid_v1;

#[charybdis_model(
    table_name = files,
    partition_keys = [user_id, id],
    clustering_keys = [],
    global_secondary_indexes = [],
    local_secondary_indexes = [],
    table_options = "",
)]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct File {
    pub user_id: Uuid,
    pub id: Uuid,
    pub name: Text,
    pub directory: Text,
    pub status: Text,
    pub created_at: Timestamp,
    pub modified_at: Timestamp,
}

impl File {
    pub fn from_request(user_id: Uuid, payload: &FileCreateRequest) -> Self {
        File {
            user_id: user_id,
            id: generate_uuid_v1().unwrap(),
            name: payload.name.to_string(),
            directory: payload.directory.to_string(),
            status: payload.status.to_string(),
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            ..Default::default()
        }
    }
}
