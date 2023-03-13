use chrono::{DateTime, Duration, Utc};
use cookie::Expiration;
use diesel::prelude::*;
use salvo::http::cookie::Cookie;
use salvo::http::StatusCode;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db::{self, lower};
use crate::models::*;
use crate::schema::*;
use crate::utils::{password, validator};
use crate::{context, AppResult, StatusInfo};

pub fn public_root(path: impl Into<String>) -> Router {
    Router::with_path(path).push(Router::with_path("login").post(login))
}
pub fn authed_root(path: impl Into<String>) -> Router {
    Router::with_path(path)
        .push(Router::with_path("logout").post(logout))
        .push(Router::with_path("refresh_token").post(refresh_token))
}
#[derive(Serialize, Deserialize, Debug)]
struct PostedLoginData {
    user: Option<String>,
    ident_name: Option<String>,
    // phone: Option<String>,
    email: Option<String>,
    password: String,
}
#[handler]
pub async fn login(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let mut pdata = parse_posted_data!(req, res, PostedLoginData);

    if let Some(user) = pdata.user {
        if let Ok(()) = validator::validate_email(&user) {
            pdata.email = Some(user);
        } else if let Ok(()) = validator::validate_ident_name(&user) {
            pdata.ident_name = Some(user);
        }
    }
    if pdata.email.is_none() && pdata.ident_name.is_none() {
        return context::render_status_json(
            res,
            StatusCode::BAD_REQUEST,
            "data_invalid",
            "data invalid",
            "user identifier is not provided",
        );
    }

    let mut conn = db::connect()?;
    let user = if let Some(ident_name) = pdata.ident_name {
        users::table
            .filter(lower(users::ident_name).eq(ident_name.to_lowercase()))
            .first::<User>(&mut conn)
            .ok()
    } else if let Some(email) = &pdata.email {
        users::table
            .filter(
                users::id.nullable().eq(emails::table
                    .filter(lower(emails::value).eq(email.to_lowercase()))
                    .select(emails::user_id)
                    .single_value()),
            )
            .first::<User>(&mut conn)
            .ok()
    } else {
        None
    };
    if user.is_none() {
        return context::render_status_json(
            res,
            StatusCode::BAD_REQUEST,
            "validate_failed",
            "validate failed",
            "Incorrect username/email or password.",
        );
    }
    let user = user.unwrap();
    if password::compare(&pdata.password, &user.password) {
        #[derive(Serialize, Debug)]
        struct ResponsedData<'a> {
            user: &'a User,
            error: Option<StatusInfo>,
            token: Option<&'a str>,
        }

        let mut data = ResponsedData {
            user: &user,
            error: None,
            token: None,
        };
        if !user.is_verified {
            data.error = Some(StatusInfo {
                code: StatusCode::BAD_REQUEST.as_u16(),
                name: "pending_verify".into(),
                summary: "user is not verified".into(),
                detail: Some("Your email address must be verified in order to continue.".into()),
                details: None,
            });
            res.render(Json(data));
            return Ok(());
        }
        
        if user.is_disabled {
            data.error = Some(StatusInfo {
                code: StatusCode::BAD_REQUEST.as_u16(),
                name: "user_disabled".into(),
                summary: "user disabled".into(),
                detail: Some("user disabled".into()),
                details: None,
            });
            res.render(Json(data));
            return Ok(());
        }
        
        match create_token(&user, &mut conn) {
            Ok(jwt_token) => {
                res.add_cookie(create_token_cookie(jwt_token.clone()));
                data.token = Some(&jwt_token);
                res.render(Json(data));
                Ok(())
            }
            Err(msg) => context::render_internal_server_error_json_with_detail(res, msg),
        }
    } else {
        context::render_status_json(
            res,
            StatusCode::BAD_REQUEST,
            "validate_failed",
            "validate failed",
            "Incorrect username/email or password.",
        )
    }
}
pub fn insert_token_to_db(
    user_id: i64,
    jwt_token: &str,
    expire: DateTime<Utc>,
    conn: &mut PgConnection,
) -> Result<usize, diesel::result::Error> {
    let new_token = NewAccessToken {
        user_id,
        kind: "web",
        value: jwt_token,
        device: None,
        name: None,
        expired_at: expire,
        updated_by: Some(user_id),
        created_by: Some(user_id),
    };
    diesel::insert_into(access_tokens::table)
        .values(&new_token)
        .execute(conn)
}
pub fn create_token_cookie(jwt_token: String) -> Cookie<'static> {
    let expires = cookie::time::OffsetDateTime::now_utc() + cookie::time::Duration::days(7);
    Cookie::build("jwt_token", jwt_token)
        .path("/")
        .domain(crate::cookie_domain())
        .secure(true)
        .http_only(false)
        .expires(Expiration::from(expires))
        .finish()
}

#[handler]
pub async fn logout(_req: &Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    if let Some(user) = context::current_user(depot) {
        let mut conn = db::connect()?;
        diesel::delete(
            access_tokens::table
                .filter(access_tokens::user_id.eq(user.id))
                .filter(access_tokens::kind.eq("web")),
        )
        .execute(&mut conn)?;
    }
    context::render_done_json(res)
}

#[handler]
pub async fn refresh_token(_req: &Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let cuser = current_user!(depot, res);
    let mut conn = db::connect()?;
    create_and_send_token(cuser, res, &mut conn)
}

pub fn create_token(user: &User, conn: &mut PgConnection) -> Result<String, String> {
    let exp = Utc::now() + Duration::days(7);
    if let Ok(jwt_token) = crate::create_jwt_token(user, &exp) {
        insert_token_to_db(user.id, &jwt_token, exp, conn).map_err(|_| "db error when insert token".to_owned())?;
        Ok(jwt_token)
    } else {
        Err("create jwt token error".into())
    }
}
pub fn create_and_send_token(user: &User, res: &mut Response, conn: &mut PgConnection) -> AppResult<()> {
    match create_token(user, conn) {
        Ok(jwt_token) => {
            #[derive(Serialize, Debug)]
            struct ResultData<'a> {
                token: &'a str,
            }
            res.add_cookie(create_token_cookie(jwt_token.clone()));
            res.render(Json(ResultData { token: &jwt_token }));
            Ok(())
        }
        Err(msg) => context::render_internal_server_error_json_with_detail(res, msg),
    }
}
