use std::env;
use std::hash::Hash;
use std::str::FromStr;
use strum_macros::{Display, EnumString};

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use jsonwebtoken as jwt;
use jsonwebtoken::EncodingKey;
use once_cell::sync::Lazy;
use salvo::http::{StatusCode, StatusError};
use serde::Serialize;
use serde_json::Value;

use crate::db::{self, lower};
use crate::models::*;
use crate::schema::*;
use crate::JwtClaims;

pub type AppResult<T> = Result<T, crate::Error>;

bitflags! {
    pub struct CalcAmountPeriod: i64 {
        const ITEM_PRICE = 0b00000001;
        const ITEM_AMOUNT = 0b00000010;
        const ORDER_AMOUNT = 0b00000100;
    }
}

static LETTER_BYTES: Lazy<Vec<u8>> = Lazy::new(|| b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".to_vec());
static DIGIT_BYTES: Lazy<Vec<u8>> = Lazy::new(|| b"0123456789".to_vec());
static CHAR_BYTES: Lazy<Vec<u8>> =
    Lazy::new(|| b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".to_vec());
static URL_SAFE_CHAR_BYTES: Lazy<Vec<u8>> =
    Lazy::new(|| b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_-".to_vec());
static PASSWORD_SAFE_CHAR_BYTES: Lazy<Vec<u8>> =
    Lazy::new(|| b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_-~*&^%$#@<>/\\[]{}+=".to_vec());
static PRESERVED_IDENT_NAMES: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "admin",
        "administrator",
        "common",
        "guest",
        "client",
        "share",
        "shared",
        "share_link",
        "sharelink",
        "root",
        "password",
        "super",
        "realm",
        "creative",
        "batch",
        "interflow",
        "trade",
        "font",
        "role",
        "user",
        "deploy",
        "campaign",
        "auth",
        "oauth",
        "order",
        "output",
        "account",
        "team",
        "product",
        "variant",
        "bundle",
        "troupe",
    ]
});


pub fn secret_key() -> String {
    env::var("SECRET_KEY").expect("SECRET_KEY must be set")
}

pub fn database_url() -> String {
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}
pub fn database_conns() -> u32 {
    env::var("DATABASE_CONNS")
        .expect("DATABASE_CONNS must be set")
        .parse::<u32>()
        .expect("DATABASE_CONNS must be i32")
}
pub fn space_path() -> String {
    env::var("SPACE_PATH").expect("SPACE_PATH must be set")
}
pub fn cookie_domain() -> String {
    env::var("COOKIE_DOMAIN").expect("COOKIE_DOMAIN must be set")
}
pub fn is_ident_name_preserved(name: &str) -> bool {
    PRESERVED_IDENT_NAMES.contains(&name)
}


#[derive(Serialize, Debug)]
pub struct StatusInfo {
    pub code: u16,
    pub name: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<String>>,
}
#[derive(Serialize, Debug)]
pub struct StatusWrap {
    pub status: StatusInfo,
}

impl StatusWrap {
    pub fn new<N, S, D>(code: StatusCode, name: N, summary: S, detail: D) -> StatusWrap
    where
        N: Into<String>,
        S: Into<String>,
        D: Into<String>,
    {
        StatusWrap {
            status: StatusInfo {
                code: code.as_u16(),
                name: name.into(),
                summary: summary.into(),
                detail: Some(detail.into()),
                details: None,
            },
        }
    }
}

#[derive(Serialize, Debug)]
pub struct ErrorWrap {
    pub error: StatusInfo,
}

impl ErrorWrap {
    pub fn new<N, S, D>(code: StatusCode, name: N, summary: S, detail: D) -> ErrorWrap
    where
        N: Into<String>,
        S: Into<String>,
        D: Into<String>,
    {
        ErrorWrap {
            error: StatusInfo {
                code: code.as_u16(),
                name: name.into(),
                summary: summary.into(),
                detail: Some(detail.into()),
                details: None,
            },
        }
    }
}

#[derive(Serialize, Debug)]
pub struct BulkErrorInfo {
    pub record_ids: Vec<i64>,
    pub name: String,
    pub summary: String,
    pub detail: String,
}

#[derive(Serialize, Debug)]
pub struct BulkResultData {
    pub done_ids: Vec<i64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<BulkErrorInfo>,
}

pub fn generate_digit_code(len: usize) -> String {
    String::from_utf8_lossy(&rand_bytes(&DIGIT_BYTES, len))
        .to_owned()
        .to_string()
}

pub fn generate_token(len: usize) -> String {
    String::from_utf8_lossy(&rand_bytes(&CHAR_BYTES, len))
        .to_owned()
        .to_string()
}
pub fn generate_url_safe_token(len: usize) -> String {
    String::from_utf8_lossy(&rand_bytes(&URL_SAFE_CHAR_BYTES, len))
        .to_owned()
        .to_string()
}
pub fn generate_ident_name(conn: &mut PgConnection) -> AppResult<String> {
    let mut ident_name = generate_token(16);
    while diesel_exists!(
        users::table.filter(lower(users::ident_name).eq(ident_name.to_lowercase())),
        conn
    )  {
        ident_name = generate_token(16);
    }
    Ok(ident_name)
}
pub fn generate_password(len: usize) -> String {
    String::from_utf8_lossy(&rand_bytes(&PASSWORD_SAFE_CHAR_BYTES, len))
        .to_owned()
        .to_string()
}
pub fn rand_bytes(bytes: &[u8], len: usize) -> Vec<u8> {
    let mut value = vec![];
    for _ in 0..len {
        value.push(bytes[rand::random::<usize>() % bytes.len()]);
    }
    value
}

pub fn create_jwt_token(user: &User, expire: &DateTime<Utc>) -> jwt::errors::Result<String> {
    let claim = JwtClaims {
        user: user.id,
        exp: expire.timestamp(),
    };
    jwt::encode(
        &jwt::Header::default(),
        &claim,
        &EncodingKey::from_secret(env::var("SECRET_KEY").expect("SECRET_KEY must be set").as_ref()),
    )
}

pub fn mask_email(email: impl AsRef<str>) -> String {
    let email = email.as_ref();
    if email.len() > 4 && email.contains('@') {
        let parts: Vec<&str> = email.split('@').collect();
        let mut left = parts[0].to_owned();
        if left.len() > 2 {
            left = format!("{}****", &left[0..2]);
        } else {
            left = format!("{}****", left);
        }

        format!("{}@{}", left, parts[1].to_owned())
    } else {
        email.into()
    }
}
pub fn mask_phone(phone: impl AsRef<str>) -> String {
    let phone = phone.as_ref();
    if phone.len() > 4 {
        format!("******{}", &phone[phone.len() - 4..])
    } else {
        phone.into()
    }
}
pub fn default_underscore() -> String {
    "_".into()
}
pub fn default_as_false() -> bool {
    false
}
pub fn default_as_true() -> bool {
    true
}
pub fn default_ffmpeg() -> String {
    "ffmpeg".into()
}

pub fn string_none_or_empty(value: &Option<String>) -> bool {
    match value {
        Some(value) => value.is_empty(),
        None => true,
    }
}

pub fn add_url_list_query(query: impl QueryDsl) -> impl QueryDsl {
    query
}

pub fn get_email_domain(email: &str) -> &str {
    email.split('@').collect::<Vec<&str>>().pop().unwrap()
}
pub fn safe_url_path(raw: &str) -> String {
    raw.replace('\\', "/").replace("../", "/")
}

pub fn bad_request_error(summary: String, detail: Option<String>) -> StatusError {
    StatusError {
        code: StatusCode::BAD_REQUEST,
        name: "Bad Request".into(),
        detail,
        summary: Some(summary),
    }
}

pub fn server_internal_error(summary: String, detail: Option<String>) -> StatusError {
    StatusError {
        code: StatusCode::INTERNAL_SERVER_ERROR,
        name: "Server Internal Error".into(),
        detail,
        summary: Some(summary),
    }
}
