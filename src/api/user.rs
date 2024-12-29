use argon2::{Argon2, PasswordHash, PasswordVerifier};
use charybdis::operations::{Find, Insert, Update};
use serde_json;
use serde_json::json;

use actix_web::HttpMessage;
use validator::Validate;

use crate::schema::user::UserResponse;
use crate::schema::user::UserUpdateRequest;
use crate::{error::ErrorMessage, model::user::User};
use crate::{
    jwt_auth,
    model::user::UsersByEmail,
    schema::user::{LoginUserRequest, LoginUserResponse, UserCreateRequest},
    utils::token::create_token,
};

use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};

use crate::config::app::AppState;
use crate::error::HttpError;

#[post("/users")]
pub async fn create_user(
    data: web::Data<AppState>,
    payload: web::Json<UserCreateRequest>,
) -> Result<impl Responder, HttpError> {
    let validated = payload.validate();

    let response = match validated {
        Ok(_) => {
            let user = UsersByEmail::find_first_by_email(payload.email.clone())
                .execute(&data.database)
                .await;
            match user {
                Ok(_) => {
                    return Err(HttpError::conflict_error(ErrorMessage::EmailExist));
                }
                Err(_) => {}
            }

            let user = User::from_request(&payload);

            user.insert().execute(&data.database).await.map_err(|err| {
                log::error!("Error creating user: {:?}", err);
                HttpError::server_error("Error during creation of a user".to_string())
            })?;

            let user_response = UserResponse {
                id: user.id,
                email: user.email,
                first_name: user.first_name,
                last_name: user.last_name,
                status: user.status,
                created_at: user.created_at,
                modified_at: user.modified_at,
            };

            HttpResponse::Ok().json(json!(user_response))
        }
        Err(err) => HttpResponse::BadRequest().json(json!(err)),
    };

    Ok(response)
}

#[put("/users/me")]
pub async fn update_user_me(
    req: HttpRequest,
    data: web::Data<AppState>,
    _: jwt_auth::JwtMiddleware,
    payload: web::Json<UserUpdateRequest>,
) -> Result<impl Responder, HttpError> {
    let ext = req.extensions();
    let user_id = ext.get::<uuid::Uuid>().unwrap();

    let validated = payload.validate();

    let response = match validated {
        Ok(_) => {
            let user = User {
                id: user_id.clone(),
                ..Default::default()
            }
            .find_by_primary_key()
            .execute(&data.database)
            .await;

            match user {
                Ok(user) => {
                    let user = User {
                        id: user_id.clone(),
                        email: user.email,
                        password_hash: user.password_hash,
                        first_name: payload.first_name.to_string(),
                        last_name: payload.last_name.to_string(),
                        status: payload.status.to_string(),
                        created_at: user.created_at,
                        modified_at: chrono::Utc::now(),
                        ..Default::default()
                    };
                    user.update().execute(&data.database).await.map_err(|e| {
                        log::error!("Error updating user: {:?}", e);
                        HttpError::server_error("Error during update of a user".to_string())
                    })?;

                    let user_response = UserResponse {
                        id: user.id,
                        email: user.email,
                        first_name: user.first_name,
                        last_name: user.last_name,
                        status: user.status,
                        created_at: user.created_at,
                        modified_at: user.modified_at,
                    };

                    HttpResponse::Ok().json(json!(user_response))
                }
                Err(_) => return Err(HttpError::not_found("User not found".to_string())),
            }
        }
        Err(err) => HttpResponse::BadRequest().json(json!(err)),
    };

    Ok(response)
}

#[get("/users/me")]
pub async fn get_user_me(
    req: HttpRequest,
    data: web::Data<AppState>,
    _: jwt_auth::JwtMiddleware,
) -> Result<impl Responder, HttpError> {
    let ext = req.extensions();
    let user_id = ext.get::<uuid::Uuid>().unwrap();

    let user = User {
        id: user_id.clone(),
        ..Default::default()
    }
    .find_by_primary_key()
    .execute(&data.database)
    .await;

    match user {
        Ok(user) => {
            let user_response = UserResponse {
                id: user.id,
                email: user.email,
                first_name: user.first_name,
                last_name: user.last_name,
                status: user.status,
                created_at: user.created_at,
                modified_at: user.modified_at,
            };

            Ok(HttpResponse::Ok().json(json!(user_response)))
        }
        Err(_) => Err(HttpError::not_found("User not found".to_string())),
    }
}

#[delete("/users/me")]
pub async fn delete_user(
    req: HttpRequest,
    data: web::Data<AppState>,
    _: jwt_auth::JwtMiddleware,
) -> Result<impl Responder, HttpError> {
    let ext = req.extensions();
    let user_id = ext.get::<uuid::Uuid>().unwrap();

    let user = User {
        id: user_id.clone(),
        ..Default::default()
    }
    .find_by_primary_key()
    .execute(&data.database)
    .await;

    match user {
        Ok(_user) => {
            User::delete_by_id(user_id.clone())
                .execute(&data.database)
                .await
                .map_err(|_| HttpError::server_error("Error deleting user".to_string()))?;

            return Ok(HttpResponse::Ok().json(json!("User deleted")));
        }
        Err(_) => return Err(HttpError::not_found("User not found".to_string())),
    }
}

#[post("/auth/login")]
async fn auth_login(
    body: web::Json<LoginUserRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let user = UsersByEmail::find_first_by_email(body.email.clone())
        .execute(&data.database)
        .await;

    match user {
        Ok(user) => {
            let parsed_hash = PasswordHash::new(&user.password_hash).unwrap();
            let is_valid = Argon2::default()
                .verify_password(body.password.as_bytes(), &parsed_hash)
                .map_or(false, |_| true);

            if !is_valid {
                return Err(HttpError::bad_request(ErrorMessage::WrongCredentials));
            }

            let token = create_token(
                &user.id.to_string().as_str(),
                data.config.app.jwt_secret.as_bytes(),
                data.config.app.jwt_maxage,
            )
            .map_err(|_| HttpError::bad_request(ErrorMessage::WrongCredentials))?;

            return Ok(HttpResponse::Ok().json(LoginUserResponse {
                id: user.id,
                email: user.email,
                first_name: user.first_name,
                last_name: user.last_name,
                status: user.status,
                created_at: user.created_at,
                modified_at: user.modified_at,
                token: token,
            }));
        }
        Err(_) => return Err(HttpError::bad_request(ErrorMessage::WrongCredentials)),
    }
}
