use crate::{ErrorWrap, StatusInfo};
use async_trait::async_trait;
use salvo::http::{StatusCode, StatusError};
use salvo::prelude::{Depot, Json, Request, Response, Writer};
use std::borrow::Cow;
use std::io;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("public: `{0}`")]
    Public(String),
    #[error("internal: `{0}`")]
    Internal(String),
    #[error("salvo internal error: `{0}`")]
    Salvo(#[from] ::salvo::Error),
    #[error("access deined")]
    AccessDeined,
    #[error("frequently request resource")]
    FrequentlyRequest,
    #[error("io: `{0}`")]
    Io(#[from] io::Error),
    #[error("utf8: `{0}`")]
    FromUtf8(#[from] FromUtf8Error),
    #[error("decoding: `{0}`")]
    Decoding(Cow<'static, str>),
    // #[error("url parse: `{0}`")]
    // UrlParse(#[from] url::ParseError),
    #[error("serde json: `{0}`")]
    SerdeJson(#[from] serde_json::error::Error),
    #[error("diesel: `{0}`")]
    Diesel(#[from] diesel::result::Error),
    #[error("zip: `{0}`")]
    Zip(#[from] zip::result::ZipError),
    // #[error("font parse: `{0}`")]
    // FontParse(#[from] ttf_parser::FaceParsingError),
    #[error("http: `{0}`")]
    HttpStatus(#[from] salvo::http::StatusError),
    #[error("http parse: `{0}`")]
    HttpParse(#[from] salvo::http::ParseError),
    // #[error("pulsar: `{0}`")]
    // Pulsar(#[from] ::pulsar::Error),
    // #[error("reqwest: `{0}`")]
    // Reqwest(#[from] reqwest::Error),
    #[error("r2d2: `{0}`")]
    R2d2(#[from] diesel::r2d2::PoolError),
    #[error("handlebars render: `{0}`")]
    HandlebarsRender(#[from] handlebars::RenderError),
    // #[error("stripe: `{0}`")]
    // Stripe(#[from] stripe::StripeError),
    // #[error("stripe ParseIdError: `{0}`")]
    // ParseIdError(#[from] stripe::ParseIdError),
    #[error("utf8: `{0}`")]
    Utf8Error(#[from] std::str::Utf8Error),
    // #[error("redis: `{0}`")]
    // Redis(#[from] redis::RedisError),
    // #[error("consumer: `{0}`")]
    // Consumer(#[from] pulsar::error::ConsumerError),
    // #[error("GlobError error: `{0}`")]
    // Glob(#[from] globwalk::GlobError),
    // #[error("image error: `{0}`")]
    // Image(#[from] image::ImageError),
    // #[error("PersistError: `{0}`")]
    // PersistError(#[from] tempfile::PersistError),
}

#[async_trait]
impl Writer for Error {
    async fn write(mut self, _req: &mut Request, depot: &mut Depot, res: &mut Response) {
        let code = match &self {
            Error::HttpStatus(e) => e.code,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        res.set_status_code(code);
        let cuser = crate::context::current_user(depot);
        if let Some(cuser) = &cuser {
            tracing::error!(error = &*self.to_string(), user_id = ?cuser.id, user_name = %cuser.ident_name, "error happened");
        } else {
            tracing::error!(error = &*self.to_string(), "error happened, user not logged in.");
        }
        let in_kernel = false;
        let data = match self {
            Error::Salvo(e) => ErrorWrap {
                error: StatusInfo {
                    code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    name: "UNKNOWN_ERROR".into(),
                    summary: "unknown error".into(),
                    detail: if in_kernel { Some(e.to_string()) } else { None },
                    details: None,
                },
            },
            Error::AccessDeined => ErrorWrap {
                error: StatusInfo {
                    code: StatusCode::FORBIDDEN.as_u16(),
                    name: "FORBIDDEN".into(),
                    summary: "access denied".into(),
                    detail: None,
                    details: None,
                },
            },
            Error::FrequentlyRequest => ErrorWrap {
                error: StatusInfo {
                    code: StatusCode::BAD_REQUEST.as_u16(),
                    name: "FREQUENTLY_REQUEST".into(),
                    summary: "frequently request resource".into(),
                    detail: None,
                    details: None,
                },
            },
            Error::Public(msg) => ErrorWrap {
                error: StatusInfo {
                    code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    name: "ERROR".into(),
                    summary: msg,
                    detail: None,
                    details: None,
                },
            },
            Error::Internal(msg) => ErrorWrap {
                error: StatusInfo {
                    code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    name: "INTERNAL_SERVER_ERROR".into(),
                    summary: msg,
                    detail: None,
                    details: None,
                },
            },
            Error::Diesel(e) => {
                tracing::error!(error = ?e, "diesel db error");
                let info = if let diesel::result::Error::NotFound = e {
                    res.set_status_code(StatusCode::NOT_FOUND);
                    StatusInfo {
                        code: StatusCode::NOT_FOUND.as_u16(),
                        name: "NOT_FOUND".into(),
                        summary: "resource not found".into(),
                        detail: if in_kernel { Some(e.to_string()) } else { None },
                        details: None,
                    }
                } else {
                    StatusInfo {
                        code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                        name: "DATABASE_ERROR".into(),
                        summary: "database error".into(),
                        detail: if in_kernel { Some(e.to_string()) } else { None },
                        details: None,
                    }
                };
                ErrorWrap { error: info }
            }
            Error::HttpStatus(e) => {
                let StatusError {
                    code,
                    name,
                    summary,
                    detail,
                } = e;
                ErrorWrap {
                    error: StatusInfo {
                        code: code.as_u16(),
                        name,
                        summary: summary.unwrap_or_else(|| "INTERNAL_SERVER_ERROR".into()),
                        detail,
                        details: None,
                    },
                }
            }
            _ => ErrorWrap {
                error: StatusInfo {
                    code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    name: "INTERNAL_SERVER_ERROR".into(),
                    summary: "internal server error".into(),
                    detail: None,
                    details: None,
                },
            },
        };
        res.render(Json(data));
    }
}
