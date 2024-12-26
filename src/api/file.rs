use charybdis::{
    operations::{Find, Insert, Update},
    types::Uuid,
};
use serde_json;
use serde_json::json;

use validator::Validate;

use crate::model::file::File;
use crate::schema::file::FileUpdateRequest;
use crate::schema::file::{FileCreateRequest, FilesResponse};

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

#[get("/v1/files")]
pub async fn get_files(data: web::Data<AppState>) -> Result<impl Responder, APIError> {
    let files = File::find_all().execute(&data.database).await;

    match files {
        Ok(files) => {
            let files = files.try_collect().await.unwrap();
            Ok(HttpResponse::Ok().json(json!(&FilesResponse { objects: files })))
        }
        Err(_) => Err(APIError::NotFound),
    }
}

#[post("/v1/files")]
pub async fn create_file(
    data: web::Data<AppState>,
    payload: web::Json<FileCreateRequest>,
) -> Result<impl Responder, APIError> {
    let validated = payload.validate();

    let response = match validated {
        Ok(_) => {
            let file = File::from_request(&payload);
            file.insert().execute(&data.database).await.unwrap();

            HttpResponse::Ok().json(json!(file))
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

            HttpResponse::Ok().json(json!(file))
        }
        Err(err) => HttpResponse::BadRequest().json(json!(err)),
    };

    Ok(response)
}

#[get("/v1/files/{id}")]
pub async fn get_file(
    file_id: Path<Uuid>,
    data: web::Data<AppState>,
) -> Result<impl Responder, APIError> {
    let file = File {
        id: file_id.into_inner(),
        ..Default::default()
    }
    .find_by_primary_key()
    .execute(&data.database)
    .await;

    match file {
        Ok(file) => Ok(HttpResponse::Ok().json(json!(file))),
        Err(_) => Err(APIError::NotFound),
    }
}

#[delete("/v1/files/{id}")]
pub async fn delete_file(
    file_id: Path<Uuid>,
    data: web::Data<AppState>,
) -> Result<impl Responder, APIError> {
    File::delete_by_id(file_id.into_inner())
        .execute(&data.database)
        .await
        .unwrap();

    return Ok(HttpResponse::Ok().json(json!("File deleted")));
}
