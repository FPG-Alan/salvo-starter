pub mod fs;
pub mod password;
pub mod validator;

use std::borrow::Cow;
use std::fmt::Write;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use once_cell::sync::Lazy;
use salvo::http::header::CONTENT_DISPOSITION;
use salvo::http::{HeaderValue, Response};
use serde_json::Value;
use uuid::Uuid;

use crate::{AppResult, Error};

pub fn hash_file_md5(path: impl AsRef<Path>) -> Result<String, std::io::Error> {
    let mut file = File::open(path.as_ref())?;
    hash_reader_md5(&mut file)
}
pub fn hash_reader_md5<R: Read>(reader: &mut R) -> Result<String, std::io::Error> {
    let mut ctx = md5::Context::new();
    io::copy(reader, &mut ctx)?;
    Ok(hash_string(&*ctx.compute()))
}
pub fn hash_str_md5(value: impl AsRef<str>) -> Result<String, std::io::Error> {
    let mut bytes = value.as_ref().as_bytes();
    hash_reader_md5(&mut bytes)
}
//https://docs.rs/crate/checksums/0.6.0/source/src/hashing/mod.rs
pub fn hash_string(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        write!(result, "{:02X}", b).unwrap();
    }
    result
}

pub fn calc_json_value_hash(value: &Value) -> AppResult<String> {
    Ok(hash_reader_md5(&mut serde_json::to_vec(value)?.as_slice())?.to_uppercase())
}

static TURE_VALUES: Lazy<Vec<&'static str>> = Lazy::new(|| vec!["true", "1", "yes", "on", "t", "y", "✓"]);
pub fn str_to_bool(v: &str) -> bool {
    TURE_VALUES.contains(&v)
}

pub fn uuid_string() -> String {
    Uuid::new_v4()
        .as_simple()
        .encode_lower(&mut Uuid::encode_buffer())
        .to_owned()
}

pub fn add_serve_file_content_disposition(
    res: &mut Response,
    file_path: impl AsRef<Path>,
    disposition_type: Option<&str>,
    attached_name: Option<&str>,
) -> AppResult<()> {
    let content_type = mime_guess::from_path(file_path.as_ref()).first_or_octet_stream();
    let disposition_type = disposition_type.unwrap_or_else(|| {
        if attached_name.is_some() {
            "attachment"
        } else {
            match (content_type.type_(), content_type.subtype()) {
                (mime::IMAGE | mime::TEXT | mime::VIDEO | mime::AUDIO, _) | (_, mime::JAVASCRIPT | mime::JSON) => {
                    "inline"
                }
                _ => "attachment",
            }
        }
    });
    let content_disposition = if disposition_type == "attachment" {
        let attached_name = match attached_name {
            Some(attached_name) => Cow::Borrowed(attached_name),
            None => file_path
                .as_ref()
                .file_name()
                .map(|file_name| file_name.to_string_lossy().to_string())
                .unwrap_or_else(|| "file".into())
                .into(),
        };
        format!("attachment; filename={}", attached_name)
            .parse::<HeaderValue>()
            .map_err(|_| Error::Internal("failed to parse http header value".into()))?
    } else {
        disposition_type
            .parse::<HeaderValue>()
            .map_err(|_| Error::Internal("failed to parse http header value".into()))?
    };
    res.headers_mut().insert(CONTENT_DISPOSITION, content_disposition);
    Ok(())
}
