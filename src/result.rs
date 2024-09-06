use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse,
};

use anyhow::anyhow;
use serde::{ser::SerializeStruct, Serialize, Serializer};

pub type HandlerResult<T> = Result<T, HandlerError>;

#[derive(Debug)]
pub struct HandlerError {
    pub err: anyhow::Error,
    pub status_code: StatusCode,
}

impl Serialize for HandlerError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("HandlerError", 2)?;
        state.serialize_field("err", &self.err.to_string())?;
        state.serialize_field("status_code", &u16::from(self.status_code))?;
        state.end()
    }
}

impl HandlerError {
    pub fn send_err<T>(err_msg: String, status_code: StatusCode) -> HandlerResult<T> {
        Err::<T, HandlerError>(Self {
            err: anyhow!("{err_msg}"),
            status_code,
        })
    }
}

impl actix_web::error::ResponseError for HandlerError {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        tracing::error!("{}", self);
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(self)
    }
}

impl<E> From<E> for HandlerError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self {
            err: value.into(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::fmt::Display for HandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.err)
    }
}

// TODO for publish redis-rpc github, need trait for result type ?
// Db connections ?
// Вынести это в features
