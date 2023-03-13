mod account;
mod auth;
mod home;
mod user;

use diesel::prelude::*;
use salvo::http::StatusCode;
use salvo::jwt_auth::{CookieFinder, HeaderFinder, JwtAuth, JwtAuthDepotExt, QueryFinder};
use salvo::prelude::*;
use salvo::routing::FlowCtrl;
use salvo::serve_static::StaticDir;
use salvo::size_limiter;
use url::Url;

use crate::db;
use crate::models::*;
use crate::schema::*;
use crate::{context, AppResult, JwtClaims};

pub fn new_jwt_auth() -> JwtAuth<JwtClaims> {
    JwtAuth::new(crate::secret_key())
        .with_finders(vec![
            Box::new(HeaderFinder::new()),
            Box::new(QueryFinder::new("jwt_token")),
            Box::new(CookieFinder::new("jwt_token")),
        ])
        .with_response_error(false)
}

#[handler]
async fn auth_final(req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
    if crate::context::current_user(depot).is_none() {
        ctrl.skip_rest();
        res.set_status_code(StatusCode::UNAUTHORIZED);
    } else {
        ctrl.call_next(req, depot, res).await;
    }
}

#[handler]
pub async fn set_user_handler(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) -> AppResult<()> {
    if let Some(data) = depot.jwt_auth_data::<crate::JwtClaims>() {
        // tracing::debug!("set_user_handler, open conn.....");
        let mut conn = db::connect()?;
        if let Ok(user) = users::table.find(data.claims.user).first::<User>(&mut conn) {
            if let Some(token) = depot.jwt_auth_token() {
                let query = access_tokens::table
                    .filter(access_tokens::value.eq(&token))
                    .filter(access_tokens::user_id.eq(user.id));
                if !user.is_disabled && diesel_exists!(query, &mut conn) {
                    depot.insert("current_user", user);
                }
            }
        }
        drop(conn);
    } else {
        let mut token: String = req
            .headers()
            .get("auth_token")
            .and_then(|v| std::str::from_utf8(v.as_bytes()).ok())
            .unwrap_or_default()
            .into();
        if token.is_empty() {
            token = req.query::<String>("auth_token").unwrap_or_default();
        }
    }
    ctrl.call_next(req, depot, res).await;
    Ok(())
}

pub fn root() -> Router {
    Router::new()
        .hoop(size_limiter::max_size(1024 * 1024 * 1024))
        .get(home::index)
        .push(Router::with_path("health").get(home::index))
        .push(auth::public_root("auth"))
        .push(account::public_root("account"))
        .push(user::public_root("users"))
        .push(
            Router::new()
                .hoop(new_jwt_auth())
                .hoop(set_user_handler)
                .hoop(auth_final)
                .push(auth::authed_root("auth"))
                .push(account::authed_root("account"))
                .push(user::authed_root("users"))
        )
        .push(
            Router::with_path("<*path>")
                .get(StaticDir::new("./static"))
        )
}