use charybdis::{
    operations::{Find, Insert, Update},
    types::Uuid,
};
use serde::Deserialize;
use serde_json;
use serde_json::json;

use validator::Validate;

use crate::schema::file::FileType;
use crate::schema::file::FileUpdateRequest;
use crate::schema::file::FilesResponse;
use crate::{client::Client, model::file::File};
use crate::{error::ErrorMessage, schema::file::FileResponse};
use crate::{jwt_auth, model::user::User, schema::file::FileCreateRequest};

use actix_web::{
    delete, get, post, put,
    web::{self, Path},
    HttpResponse, Responder,
};

use crate::config::app::AppState;
use crate::error::HttpError;

#[derive(Deserialize)]
pub struct PaginationQuery {
    last_id: Option<Uuid>, // Adjust type based on your ID field type
    limit: Option<i32>,    // Optional limit parameter
}

#[get("/files")]
pub async fn get_files(
    data: web::Data<AppState>,
    jwt: jwt_auth::JwtMiddleware,
    query: web::Query<PaginationQuery>,
) -> Result<impl Responder, HttpError> {
    let user = jwt.get_user(&data.database).await?;

    User::find_first_by_id(user.id.clone())
        .execute(&data.database)
        .await
        .map_err(|_| HttpError::not_found(ErrorMessage::FileNotFound))?;

    match query.last_id.clone() {
        Some(last_id) => {
            let files = File::find(
                "SELECT * FROM files WHERE user_id = ? AND id > ? LIMIT ?",
                (
                    user.id.clone(),
                    last_id.clone(),
                    query.limit.unwrap_or(100).min(100),
                ),
            )
            .execute(&data.database)
            .await;

            match files {
                Ok(files) => {
                    let files = files.try_collect().await.map_err(|e| {
                        log::error!("Error fetching files: {:?}", e);
                        HttpError::server_error(ErrorMessage::ServerError)
                    })?;

                    Ok(HttpResponse::Ok().json(json!(&FilesResponse { objects: files })))
                }
                Err(_) => Err(HttpError::server_error(ErrorMessage::ServerError)),
            }
        }
        None => {
            let files = File::find(
                "SELECT * FROM files WHERE user_id = ? LIMIT ?",
                (user.id.clone(), query.limit.unwrap_or(100).min(100)),
            )
            .execute(&data.database)
            .await;

            match files {
                Ok(files) => {
                    let files = files.try_collect().await.map_err(|e| {
                        log::error!("Error fetching files: {:?}", e);
                        HttpError::server_error(ErrorMessage::ServerError)
                    })?;
                    Ok(HttpResponse::Ok().json(json!(&FilesResponse { objects: files })))
                }
                Err(_) => Err(HttpError::server_error(ErrorMessage::ServerError)),
            }
        }
    }
}

#[post("/files")]
pub async fn create_file(
    data: web::Data<AppState>,
    jwt: jwt_auth::JwtMiddleware,
    client: web::Data<Client>,
    payload: web::Json<FileCreateRequest>,
) -> Result<impl Responder, HttpError> {
    let user = jwt.get_user(&data.database).await?;

    User::find_first_by_id(user.id.clone())
        .execute(&data.database)
        .await
        .map_err(|_| HttpError::not_found(ErrorMessage::FileNotFound))?;

    let validated = payload.validate();

    let response = match validated {
        Ok(_) => {
            let file = File::from_request(user.id.clone(), &payload);
            file.insert().execute(&data.database).await.map_err(|err| {
                log::error!("Error fetching files: {:?}", err);
                HttpError::server_error(ErrorMessage::ServerError)
            })?;

            let mut file_response = FileResponse {
                id: file.id,
                name: file.name,
                directory: file.directory,
                file_type: file.file_type,
                status: file.status,
                created_at: file.created_at,
                modified_at: file.modified_at,
                presigned_url: None,
                upload_presigned_url: None,
            };

            if file_response.file_type == FileType::FILE.to_string() {
                // join directory and name to create object path
                let object_path = std::path::Path::new("memora")
                    .join(&file_response.directory)
                    .join(&file_response.name);

                let upload_presigned_url = client
                    .get_upload_presigned_url(&object_path.to_str().unwrap(), 60 * 60 * 24)
                    .await;

                match upload_presigned_url {
                    Ok(url) => {
                        log::info!("Presigned UPLOAD URL: {:?}", url);
                        file_response.upload_presigned_url = Some(url);
                    }
                    Err(err) => {
                        log::error!("Error generating presigned URL: {}", err);
                    }
                }
            }

            HttpResponse::Ok().json(json!(file_response))
        }
        Err(err) => HttpResponse::BadRequest().json(json!(err)),
    };

    Ok(response)
}

#[put("/files/{id}")]
pub async fn update_file(
    file_id: Path<Uuid>,
    data: web::Data<AppState>,
    jwt: jwt_auth::JwtMiddleware,
    payload: web::Json<FileUpdateRequest>,
) -> Result<impl Responder, HttpError> {
    let user = jwt.get_user(&data.database).await?;

    User::find_first_by_id(user.id.clone())
        .execute(&data.database)
        .await
        .map_err(|_| HttpError::not_found(ErrorMessage::FileNotFound))?;

    let validated = payload.validate();

    let response = match validated {
        Ok(_) => {
            let file = File {
                user_id: user.id.clone(),
                id: file_id.into_inner(),
                name: payload.name.to_string(),
                directory: payload.directory.to_string(),
                file_type: payload.file_type.to_string(),
                status: payload.status.to_string(),
                created_at: payload.created_at,
                modified_at: payload.modified_at,
                ..Default::default()
            };
            file.update().execute(&data.database).await.map_err(|e| {
                log::error!("Error updating file: {:?}", e);
                HttpError::server_error(ErrorMessage::ServerError)
            })?;

            let file_response = FileResponse {
                id: file.id,
                name: file.name,
                directory: file.directory,
                file_type: file.file_type,
                status: file.status,
                created_at: file.created_at,
                modified_at: file.modified_at,
                presigned_url: None,
                upload_presigned_url: None,
            };

            HttpResponse::Ok().json(json!(file_response))
        }
        Err(err) => HttpResponse::BadRequest().json(json!(err)),
    };

    Ok(response)
}

#[get("/files/{id}")]
pub async fn get_file(
    file_id: Path<Uuid>,
    data: web::Data<AppState>,
    jwt: jwt_auth::JwtMiddleware,
    client: web::Data<Client>,
) -> Result<impl Responder, HttpError> {
    let user = jwt.get_user(&data.database).await?;

    User::find_first_by_id(user.id.clone())
        .execute(&data.database)
        .await
        .map_err(|_| HttpError::not_found(ErrorMessage::FileNotFound))?;

    let file = File {
        user_id: user.id.clone(),
        id: file_id.into_inner(),
        ..Default::default()
    }
    .find_by_primary_key()
    .execute(&data.database)
    .await;

    match file {
        Ok(file) => {
            let mut file_response = FileResponse {
                id: file.id,
                name: file.name.clone(),
                directory: file.directory.clone(),
                file_type: file.file_type.clone(),
                status: file.status,
                created_at: file.created_at,
                modified_at: file.modified_at,
                presigned_url: None,
                upload_presigned_url: None,
            };

            if file_response.file_type == FileType::FILE.to_string() {
                // join directory and name to create object path
                let object_path = std::path::Path::new("memora")
                    .join(&file.directory)
                    .join(&file.name);

                let presigned_url = client
                    .get_presigned_url(&object_path.to_str().unwrap(), 60 * 60 * 24)
                    .await;

                match presigned_url {
                    Ok(url) => {
                        log::info!("Presigned UPLOAD URL: {:?}", url);
                        file_response.upload_presigned_url = Some(url);
                    }
                    Err(err) => {
                        log::error!("Error generating presigned URL: {}", err);
                    }
                }
            }

            Ok(HttpResponse::Ok().json(json!(file_response)))
        }
        Err(_) => Err(HttpError::not_found(ErrorMessage::FileNotFound)),
    }
}

#[delete("/files/{id}")]
pub async fn delete_file(
    file_id: Path<Uuid>,
    data: web::Data<AppState>,
    jwt: jwt_auth::JwtMiddleware,
    client: web::Data<Client>,
) -> Result<impl Responder, HttpError> {
    let user = jwt.get_user(&data.database).await?;

    User::find_first_by_id(user.id.clone())
        .execute(&data.database)
        .await
        .map_err(|_| HttpError::not_found(ErrorMessage::FileNotFound))?;

    let file = File {
        user_id: user.id.clone(),
        id: file_id.clone(),
        ..Default::default()
    }
    .find_by_primary_key()
    .execute(&data.database)
    .await;

    match file {
        Ok(file) => {
            if file.file_type == FileType::FILE.to_string() {
                // join directory and name to create object path
                let object_path = std::path::Path::new("memora")
                    .join(&file.directory)
                    .join(&file.name);

                client
                    .delete_object(&object_path.to_str().unwrap())
                    .await
                    .map_err(|_| HttpError::server_error(ErrorMessage::ServerError))?;
            }
        }
        Err(_) => return Err(HttpError::not_found(ErrorMessage::FileNotFound)),
    }

    File::delete_by_user_id_and_id(user.id.clone(), file_id.into_inner())
        .execute(&data.database)
        .await
        .map_err(|_| HttpError::server_error(ErrorMessage::ServerError))?;

    return Ok(HttpResponse::Ok().json(json!("File deleted")));
}

#[get("/files/{directory:.*}")]
pub async fn get_files_by_directory(
    directory: Path<String>,
    data: web::Data<AppState>,
    jwt: jwt_auth::JwtMiddleware,
    query: web::Query<PaginationQuery>,
) -> Result<impl Responder, HttpError> {
    let user = jwt.get_user(&data.database).await?;

    log::info!("directory: {:?}", directory);

    User::find_first_by_id(user.id.clone())
        .execute(&data.database)
        .await
        .map_err(|_| HttpError::not_found(ErrorMessage::FileNotFound))?;

    match query.last_id.clone() {
        Some(last_id) => {
            let files = File::find(
                "SELECT * FROM files_by_directory WHERE user_id = ? AND directory = ? AND id > ? LIMIT ?",
                (
                    user.id.clone(),
                    directory.into_inner(),
                    last_id.clone(),
                    query.limit.unwrap_or(100).min(100),
                ),
            )
            .execute(&data.database)
            .await;

            match files {
                Ok(files) => {
                    let files = files.try_collect().await.map_err(|e| {
                        log::error!("Error fetching files: {:?}", e);
                        HttpError::server_error(ErrorMessage::ServerError)
                    })?;

                    Ok(HttpResponse::Ok().json(json!(&FilesResponse { objects: files })))
                }
                Err(_) => Err(HttpError::server_error(ErrorMessage::ServerError)),
            }
        }
        None => {
            let files = File::find(
                "SELECT * FROM files_by_directory WHERE user_id = ? AND directory = ? LIMIT ?",
                (
                    user.id.clone(),
                    directory.clone(),
                    query.limit.unwrap_or(100).min(100),
                ),
            )
            .execute(&data.database)
            .await;

            match files {
                Ok(files) => {
                    let files = files.try_collect().await.map_err(|e| {
                        log::error!("Error fetching files: {:?}", e);
                        HttpError::server_error(ErrorMessage::ServerError)
                    })?;
                    Ok(HttpResponse::Ok().json(json!(&FilesResponse { objects: files })))
                }
                Err(_) => Err(HttpError::server_error(ErrorMessage::ServerError)),
            }
        }
    }
}
