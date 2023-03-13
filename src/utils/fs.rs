use std::ffi::OsStr;
use std::fs::{self, File};
use std::io;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use salvo::fs::NamedFile;
use salvo::http::form::FilePart;
use salvo::http::HeaderMap;
use salvo::prelude::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use textnonce::TextNonce;

use crate::{AppResult};

pub async fn smart_upload_files(
    req: &mut Request,
    field: Option<&str>,
    store_dir: impl AsRef<str>,
    unique: bool,
) -> AppResult<UploadedData> {
    if is_single_zip_file_upload(req, field).await {
        let file = match field {
            Some(field) => req.file(field).await,
            None => match req.form_data().await {
                Ok(form_data) => Some(form_data.files.iter().next().unwrap().1),
                Err(_) => None,
            },
        };
        match file {
            Some(file) => unzip_uploaded_file(file, store_dir.as_ref(), unique).await,
            None => Err(crate::Error::Internal("no file found".to_owned())),
        }
    } else {
        upload_files(req, store_dir, unique).await
    }
}

pub async fn is_single_zip_file_upload(req: &mut Request, field: Option<&str>) -> bool {
    let form_data = match req.form_data().await {
        Ok(form_data) => form_data,
        Err(e) => {
            tracing::error!(error=?e, "error when get form data");
            return false;
        }
    };
    match field {
        Some(field) => {
            if form_data.files.get_vec(field).unwrap_or(&vec![]).len() == 1 {
                return form_data
                    .files
                    .get(field)
                    .unwrap()
                    .name()
                    .unwrap_or_default()
                    .ends_with(".zip");
            }
        }
        None => {
            if form_data.files.len() == 1 {
                for (_, values) in form_data.files.iter_all() {
                    if values.len() == 1 {
                        values[0].name().unwrap_or_default().ends_with(".zip");
                    }
                }
            }
        }
    }
    false
}
pub fn is_safe_dir_path(dir_path: &str) -> bool {
    !dir_path.contains('.') && !dir_path.contains(':') && !dir_path.contains('\\') && !dir_path.starts_with('/')
}
#[derive(Serialize, Debug)]
pub struct UploadedData {
    pub base: String,
    pub files: Vec<UploadedFile>,
}
#[derive(Serialize, Debug)]
pub struct UploadedFile {
    pub key: String,
    pub path: String,
    pub hash: Option<String>,
}
pub async fn upload_files(req: &mut Request, store_dir: impl AsRef<str>, unique: bool) -> AppResult<UploadedData> {
    let store_dir = store_dir.as_ref();
    let form_data = match req.form_data().await {
        Ok(form_data) => form_data,
        Err(e) => {
            tracing::error!( error = ?e, "form data error");
            return Err(crate::Error::Internal("form data error".into()));
        }
    };
    let mut result = UploadedData {
        base: store_dir.to_owned(),
        files: vec![],
    };
    for (_, values) in form_data.files.iter_all() {
        for value in values {
            let oname = value.name().unwrap_or_default().trim_start_matches('/').to_owned();
            if oname.is_empty() {
                continue;
            }
            let ext: String = crate::utils::fs::get_file_ext(Path::new(&oname));
            if ext.is_empty() {
                continue;
            }
            let hash = if unique {
                Some(super::hash_file_md5(&value.path())?)
            } else {
                None
            };
            let dir_path = join_path!(&crate::space_path(), &store_dir);
            fs::create_dir_all(&dir_path).ok();
            let fname = if unique {
                format!("{}.{}", hash.as_ref().unwrap(), &ext)
            } else {
                oname.clone()
            };
            let fpath = Path::new(&dir_path).join(&fname);
            fs::copy(&value.path(), &fpath)?;
            result.files.push(UploadedFile {
                key: oname,
                path: fname,
                hash,
            });
        }
    }
    Ok(result)
}
pub struct TempPath(String);
impl TempPath {
    pub fn new(path: impl Into<String>) -> Self {
        TempPath(path.into())
    }
}
impl Drop for TempPath {
    fn drop(&mut self) {
        ::std::fs::remove_dir_all(&self.0).ok();
    }
}
pub async fn unzip_uploaded_file(
    file_part: &FilePart,
    store_dir: impl AsRef<str>,
    unique: bool,
) -> AppResult<UploadedData> {
    Ok(UploadedData {
        base: store_dir.as_ref().into(),
        files: unzip_file(file_part.path(), store_dir.as_ref(), unique).await?,
    })
}
fn file_name_sanitized(file_name: &str) -> ::std::path::PathBuf {
    let no_null_filename = match file_name.find('\0') {
        Some(index) => &file_name[0..index],
        None => file_name,
    }
    .to_string();

    // zip files can contain both / and \ as separators regardless of the OS
    // and as we want to return a sanitized PathBuf that only supports the
    // OS separator let's convert incompatible separators to compatible ones
    let separator = ::std::path::MAIN_SEPARATOR;
    let opposite_separator = match separator {
        '/' => '\\',
        _ => '/',
    };
    let filename = no_null_filename.replace(&opposite_separator.to_string(), &separator.to_string());

    ::std::path::Path::new(&filename)
        .components()
        .filter(|component| matches!(*component, ::std::path::Component::Normal(..)))
        .fold(::std::path::PathBuf::new(), |mut path, ref cur| {
            path.push(cur.as_os_str());
            path
        })
}
pub async fn unzip_file(src: impl AsRef<Path>, dest: impl AsRef<str>, unique: bool) -> AppResult<Vec<UploadedFile>> {
    let file = fs::File::open(src.as_ref())?;
    let dest = dest.as_ref();
    let mut archive = zip::ZipArchive::new(file)?;

    let mut data = Vec::with_capacity(archive.len());
    let tmpath = if unique {
        Some(TempPath::new(join_path!(
            dest,
            TextNonce::sized_urlsafe(32).unwrap().into_string()
        )))
    } else {
        None
    };
    let mut all_files = Vec::with_capacity(archive.len());
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let fname = &*file.name().to_owned().trim_start_matches('/').to_string();
        let sname = file_name_sanitized(fname);
        let ext = get_file_ext(sname.as_path());
        let mut out_path = if unique {
            join_path!(crate::space_path(), &tmpath.as_ref().unwrap().0, &sname)
        } else {
            join_path!(crate::space_path(), dest, &sname)
        };
        // if sname.to_str().unwrap_or_default().starts_with('.') || ext.is_empty() {
        //     //skip hide files and file without ext
        //     continue;
        // }
        if fname.ends_with('/') {
            // println!("File {} extracted to \"{:#?}\"", i, out_path.to_str());
            fs::create_dir_all(&out_path)?;
        } else {
            // println!("File {} extracted to \"{:#?}\" ({} bytes)", i, out_path.to_str(), file.size());
            if let Some(p) = Path::new(&out_path).parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = fs::File::create(&out_path)?;
            io::copy(&mut file, &mut outfile)?;
            let hash = super::hash_file_md5(&out_path)?;
            if !unique {
                data.push(UploadedFile {
                    key: fname.to_string(),
                    path: fname.to_string(),
                    hash: Some(hash),
                });
            } else {
                let hfname = format!("{}.{}", &hash, &ext);
                let new_out_path = join_path!(crate::space_path(), dest, &hfname);
                fs::rename(&out_path, &new_out_path)?;
                out_path = new_out_path;
                data.push(UploadedFile {
                    key: fname.to_string(),
                    path: hfname,
                    hash: Some(hash),
                });
            }
            all_files.push(out_path);
        }
        // Get and Set permissions
        // #[cfg(unix)]
        // {
        //     use std::os::unix::fs::PermissionsExt;

        //     if let Some(mode) = file.unix_mode() {
        //         fs::set_permissions(&out_path, fs::Permissions::from_mode(mode))?;
        //     }
        // }
    }

    Ok(data)
}

pub fn get_file_ext<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .to_lowercase()
}

pub fn read_json<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> AppResult<T> {
    let file = File::open(path.as_ref())?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader::<_, T>(reader)?)
}

pub fn write_json<P: AsRef<Path>, C: Serialize>(path: P, contents: C, pretty: bool) -> AppResult<()> {
    std::fs::create_dir_all(get_parent_dir(path.as_ref()))?;
    if pretty {
        std::fs::write(path, serde_json::to_vec_pretty(&contents)?)?;
    } else {
        std::fs::write(path, serde_json::to_vec(&contents)?)?;
    }
    Ok(())
}

pub fn get_parent_dir<T>(path: T) -> PathBuf
where
    T: AsRef<Path>,
{
    let mut parent_dir = path.as_ref().to_owned();
    parent_dir.pop();
    parent_dir
}

pub fn is_image_ext(ext: &str) -> bool {
    ["gif", "jpg", "jpeg", "webp", "avif", "png", "svg"].contains(&ext)
}
pub fn is_video_ext(ext: &str) -> bool {
    ["mp4", "mov", "avi", "wmv", "webm"].contains(&ext)
}
pub fn is_audio_ext(ext: &str) -> bool {
    ["mp3", "flac", "wav", "aac", "ogg", "alac", "wma", "m4a"].contains(&ext)
}
pub fn is_font_ext(ext: &str) -> bool {
    ["ttf", "otf", "woff", "woff2"].contains(&ext)
}

pub async fn send_local_file(
    key: impl AsRef<str>,
    req_headers: &HeaderMap,
    res: &mut Response,
    attched_name: Option<&str>,
) {
    if attched_name.is_some() {
        if let Err(e) = super::add_serve_file_content_disposition(res, key.as_ref(), None, attched_name) {
            tracing::error!("add_serve_file_content_disposition error: {}", e);
        }
    }
    let file_path = join_path!(crate::space_path(), key.as_ref());
    if Path::new(&file_path).exists() {
        NamedFile::send_file(file_path, req_headers, res).await;
    }
}