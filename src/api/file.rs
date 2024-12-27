use charybdis::{
    operations::{Find, Insert, Update},
    types::Uuid,
};
use serde::Deserialize;
use serde_json;
use serde_json::json;

use validator::Validate;

use crate::schema::file::FileCreateRequest;
use crate::schema::file::FileResponse;
use crate::schema::file::FileUpdateRequest;
use crate::schema::file::FilesResponse;
use crate::{client::Client, model::file::File};

use actix_web::{
    delete,
    error::ResponseError,
    get,
    http::StatusCode,
    post, put,
    web::{self, Path},
    HttpResponse, Responder,
};
use derive_more::{Display, Error};

use crate::config::app::AppState;

#[derive(serde::Serialize)]
pub struct Error {
    pub error: String,
}

#[derive(Debug, Display, Error)]
pub enum APIError {
    NotFound,
}

impl ResponseError for APIError {
    fn status_code(&self) -> StatusCode {
        match self {
            APIError::NotFound => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(
            serde_json::to_string(&Error {
                error: self.to_string(),
            })
            .unwrap(),
        )
    }
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    last_id: Option<Uuid>, // Adjust type based on your ID field type
    limit: Option<i32>,    // Optional limit parameter
}

#[get("/v1/files")]
pub async fn get_files(
    data: web::Data<AppState>,
    query: web::Query<PaginationQuery>,
) -> Result<impl Responder, APIError> {
    match query.last_id.clone() {
        Some(last_id) => {
            let files = File::find(
                "SELECT * FROM files WHERE token(id) > token(?) LIMIT ?",
                (last_id.clone(), query.limit.unwrap_or(100).min(100)),
            )
            .execute(&data.database)
            .await;

            match files {
                Ok(files) => {
                    let files = files.try_collect().await.unwrap();

                    Ok(HttpResponse::Ok().json(json!(&FilesResponse { objects: files })))
                }
                Err(_) => Err(APIError::NotFound),
            }
        }
        None => {
            let files = File::find(
                "SELECT * FROM files LIMIT ?",
                (query.limit.unwrap_or(100).min(100),),
            )
            .execute(&data.database)
            .await;

            match files {
                Ok(files) => {
                    let files = files.try_collect().await.unwrap();
                    Ok(HttpResponse::Ok().json(json!(&FilesResponse { objects: files })))
                }
                Err(_) => Err(APIError::NotFound),
            }
        }
    }
}

#[post("/v1/files")]
pub async fn create_file(
    data: web::Data<AppState>,
    client: web::Data<Client>,
    payload: web::Json<FileCreateRequest>,
) -> Result<impl Responder, APIError> {
    let validated = payload.validate();

    let response = match validated {
        Ok(_) => {
            let file = File::from_request(&payload);
            file.insert().execute(&data.database).await.unwrap();

            // join directory and name to create object path
            let object_path = std::path::Path::new("memora")
                .join(&file.directory)
                .join(&file.name);

            let upload_presigned_url = client
                .get_upload_presigned_url(&object_path.to_str().unwrap(), 60 * 60 * 24)
                .await;

            let mut file_response = FileResponse {
                id: file.id,
                name: file.name,
                directory: file.directory,
                status: file.status,
                created_at: file.created_at,
                modified_at: file.modified_at,
                presigned_url: None,
                upload_presigned_url: None,
            };

            match upload_presigned_url {
                Ok(url) => {
                    log::info!("Presigned UPLOAD URL: {:?}", url);
                    file_response.upload_presigned_url = Some(url);
                }
                Err(err) => {
                    log::error!("Error generating presigned URL: {}", err);
                }
            }

            HttpResponse::Ok().json(json!(file_response))
        }
        Err(err) => HttpResponse::BadRequest().json(json!(err)),
    };

    Ok(response)
}

#[put("/v1/files/{id}")]
pub async fn update_file(
    file_id: Path<Uuid>,
    data: web::Data<AppState>,
    payload: web::Json<FileUpdateRequest>,
) -> Result<impl Responder, APIError> {
    let validated = payload.validate();

    let response = match validated {
        Ok(_) => {
            let file = File {
                id: file_id.into_inner(),
                name: payload.name.to_string(),
                directory: payload.directory.to_string(),
                status: payload.status.to_string(),
                created_at: payload.created_at,
                modified_at: payload.modified_at,
                ..Default::default()
            };
            file.update().execute(&data.database).await.unwrap();

            let file_response = FileResponse {
                id: file.id,
                name: file.name,
                directory: file.directory,
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

#[get("/v1/files/{id}")]
pub async fn get_file(
    file_id: Path<Uuid>,
    data: web::Data<AppState>,
    client: web::Data<Client>,
) -> Result<impl Responder, APIError> {
    let file = File {
        id: file_id.into_inner(),
        ..Default::default()
    }
    .find_by_primary_key()
    .execute(&data.database)
    .await;

    match file {
        Ok(file) => {
            // join directory and name to create object path
            let object_path = std::path::Path::new("memora")
                .join(&file.directory)
                .join(&file.name);

            let presigned_url = client
                .get_presigned_url(&object_path.to_str().unwrap(), 60 * 60 * 24)
                .await;

            let mut file_response = FileResponse {
                id: file.id,
                name: file.name,
                directory: file.directory,
                status: file.status,
                created_at: file.created_at,
                modified_at: file.modified_at,
                presigned_url: None,
                upload_presigned_url: None,
            };

            match presigned_url {
                Ok(url) => {
                    log::info!("Presigned UPLOAD URL: {:?}", url);
                    file_response.upload_presigned_url = Some(url);
                }
                Err(err) => {
                    log::error!("Error generating presigned URL: {}", err);
                }
            }

            Ok(HttpResponse::Ok().json(json!(file_response)))
        }
        Err(_) => Err(APIError::NotFound),
    }
}

#[delete("/v1/files/{id}")]
pub async fn delete_file(
    file_id: Path<Uuid>,
    data: web::Data<AppState>,
    client: web::Data<Client>,
) -> Result<impl Responder, APIError> {
    let file = File {
        id: file_id.clone(),
        ..Default::default()
    }
    .find_by_primary_key()
    .execute(&data.database)
    .await;

    match file {
        Ok(file) => {
            // join directory and name to create object path
            let object_path = std::path::Path::new("memora")
                .join(&file.directory)
                .join(&file.name);

            client
                .delete_object(&object_path.to_str().unwrap())
                .await
                .map_err(|_| APIError::NotFound)?;
        }
        Err(_) => return Err(APIError::NotFound),
    }

    File::delete_by_id(file_id.into_inner())
        .execute(&data.database)
        .await
        .unwrap();

    return Ok(HttpResponse::Ok().json(json!("File deleted")));
}
