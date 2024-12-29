use crate::schema::user::UserCreateRequest;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::Argon2;
use argon2::PasswordHasher;
use charybdis::macros::charybdis_model;
use charybdis::macros::charybdis_view_model;
use charybdis::types::{Text, Timestamp, Uuid};
use serde::{Deserialize, Serialize};

use crate::utils::node::generate_uuid_v1;

#[charybdis_model(
    table_name = users,
    partition_keys = [id],
    clustering_keys = [],
    global_secondary_indexes = [],
    local_secondary_indexes = [],
    table_options = "",
)]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct User {
    pub id: Uuid,
    pub email: Text,
    pub password_hash: Text,
    pub first_name: Text,
    pub last_name: Text,
    pub status: Text,
    pub created_at: Timestamp,
    pub modified_at: Timestamp,
}

impl User {
    pub fn from_request(payload: &UserCreateRequest) -> Self {
        let salt = SaltString::generate(&mut OsRng);
        let hashed_password = Argon2::default()
            .hash_password(payload.password.as_bytes(), &salt)
            .expect("Error while hashing password")
            .to_string();

        User {
            id: generate_uuid_v1().unwrap(),
            email: payload.email.to_string(),
            password_hash: hashed_password,
            first_name: payload.first_name.to_string(),
            last_name: payload.last_name.to_string(),
            status: "active".to_string(),
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            ..Default::default()
        }
    }
}

#[charybdis_view_model(
    table_name=users_by_email,
    base_table=users,
    partition_keys=[email],
    clustering_keys=[id]
)]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct UsersByEmail {
    pub id: Uuid,
    pub email: Text,
    pub password_hash: Text,
    pub first_name: Text,
    pub last_name: Text,
    pub status: Text,
    pub created_at: Timestamp,
    pub modified_at: Timestamp,
}
