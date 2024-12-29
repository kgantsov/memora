use charybdis::types::{Text, Timestamp, Uuid};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::model::user::User;

#[derive(Serialize, Debug, Validate)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: Text,
    pub first_name: Text,
    pub last_name: Text,
    pub status: Text,
    pub created_at: Timestamp,
    pub modified_at: Timestamp,
}

#[derive(Deserialize, Debug, Validate)]
pub struct UserCreateRequest {
    pub email: Text,
    pub password: Text,
    pub first_name: Text,
    pub last_name: Text,
}

#[derive(Deserialize, Debug, Validate)]
pub struct UserUpdateRequest {
    pub first_name: Text,
    pub last_name: Text,
    pub status: Text,
}

#[derive(Serialize)]
pub struct UsersResponse {
    pub objects: Vec<User>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

#[derive(Deserialize, Debug, Validate)]
pub struct LoginUserRequest {
    pub email: Text,
    pub password: Text,
}

#[derive(Serialize, Debug, Validate)]
pub struct LoginUserResponse {
    pub id: Uuid,
    pub email: Text,
    pub first_name: Text,
    pub last_name: Text,
    pub status: Text,
    pub created_at: Timestamp,
    pub modified_at: Timestamp,
    pub token: Text,
}
