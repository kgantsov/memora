use core::fmt;
use std::future::{ready, Ready};

use actix_web::error::ErrorUnauthorized;
use actix_web::{dev::Payload, Error as ActixWebError};
use actix_web::{http, web, FromRequest, HttpMessage, HttpRequest};
use jsonwebtoken::{decode, DecodingKey, Validation};
use scylla::CachingSession;
use serde::Serialize;

use crate::error::{ErrorMessage, HttpError};
use crate::model::user::User;
use crate::schema::user::TokenClaims;
use crate::AppState;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    status: String,
    message: String,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

pub struct JwtMiddleware {
    pub user_id: uuid::Uuid,
}

impl FromRequest for JwtMiddleware {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let data = req.app_data::<web::Data<AppState>>().unwrap();

        let token = req
            .headers()
            .get(http::header::AUTHORIZATION)
            .map(|h| h.to_str().unwrap().split_at(7).1.to_string());

        if token.is_none() {
            let json_error = ErrorResponse {
                status: "fail".to_string(),
                message: ErrorMessage::TokenNotProvided.to_string(),
            };
            return ready(Err(ErrorUnauthorized(json_error)));
        }

        let claims = match decode::<TokenClaims>(
            &token.unwrap(),
            &DecodingKey::from_secret(data.config.app.jwt_secret.as_ref()),
            &Validation::default(),
        ) {
            Ok(c) => c.claims,
            Err(_) => {
                let json_error = ErrorResponse {
                    status: "fail".to_string(),
                    message: ErrorMessage::InvalidToken.to_string(),
                };
                return ready(Err(ErrorUnauthorized(json_error)));
            }
        };

        let user_id = uuid::Uuid::parse_str(claims.sub.as_str()).unwrap();
        req.extensions_mut()
            .insert::<uuid::Uuid>(user_id.to_owned());

        ready(Ok(JwtMiddleware { user_id }))
    }
}

impl JwtMiddleware {
    pub async fn get_user(&self, session: &CachingSession) -> Result<User, HttpError> {
        let user = User::find_first_by_id(self.user_id.clone())
            .execute(session)
            .await
            .map_err(|_| HttpError::not_found(ErrorMessage::UserNoLongerExist))?;

        Ok(user)
    }
}
