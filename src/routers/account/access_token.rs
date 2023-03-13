use chrono::{Duration, Utc};
use diesel::prelude::*;
use salvo::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

use crate::db;
use crate::models::*;
use crate::schema::*;
use crate::utils::validator;
use crate::{context, AppResult};

#[handler]
pub async fn delete(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let mut conn = db::connect()?;
    delete_record!(
        req,
        depot,
        res,
        access_tokens,
        AccessToken,
        db::delete_access_token,
        users,
        User,
        user_id,
        "edit",
        &mut conn
    );
    Ok(())
}
#[handler]
pub async fn bulk_delete(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let mut conn = db::connect()?;
    bulk_delete_records!(
        req,
        depot,
        res,
        access_tokens,
        AccessToken,
        db::delete_access_token,
        users,
        User,
        user_id,
        "edit",
        &mut conn
    );
    Ok(())
}

#[handler]
pub async fn list(_req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let cuser = current_user!(depot, res);
    let query = access_tokens::table
        .filter(access_tokens::user_id.eq(cuser.id))
        .filter(access_tokens::kind.eq("api"));
    let mut conn = db::connect()?;
    res.render(Json(query.get_results::<AccessToken>(&mut conn)?));
    Ok(())
}

#[handler]
pub async fn create(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        user_id: i64,
        #[serde(default)]
        name: String,
        #[serde(default)]
        value: String,
        device: Option<String>,
    }
    let pdata = parse_posted_data!(req, res, PostedData);
    if pdata.name.is_empty() {
        return context::render_parse_param_error_json_with_detail(res, "name is not provider");
    }
    if let Err(e) = validator::validate_generic_name(&pdata.name) {
        return context::render_parse_param_error_json_with_detail(res, e);
    }
    let cuser = current_user!(depot, res);
    let exp = Utc::now() + Duration::days(7);
    let jwt_token = crate::create_jwt_token(cuser, &exp);
    if jwt_token.is_err() {
        return context::render_internal_server_error_json_with_detail(res, "create jwt token error");
    }
    let jwt_token = jwt_token.unwrap();
    let mut conn = db::connect()?;
    let output = conn.transaction::<_, crate::Error, _>(|conn| {
        let query = access_tokens::table
            .filter(access_tokens::user_id.eq(cuser.id))
            .filter(access_tokens::name.eq(&pdata.name));
        if diesel_exists!(query, conn) {
            return Err(StatusError::conflict()
                .with_summary("token conflict")
                .with_detail("this name is already taken, please try another.")
                .into());
        }
        let token = NewAccessToken {
            user_id: cuser.id,
            name: Some(&pdata.name),
            value: jwt_token.split('.').collect::<Vec<&str>>()[2],
            kind: "api",
            device: None,
            expired_at: exp,
            updated_by: Some(cuser.id),
            created_by: Some(cuser.id),
        };
        diesel::insert_into(access_tokens::table).values(&token).execute(conn)?;
        let mut output = HashMap::new();
        output.insert("value", token.value);
        Ok(output)
    })?;
    res.render(Json(output));
    Ok(())
}

#[handler]
pub async fn update(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        user_id: i64,
        #[serde(default)]
        name: String,
        #[serde(default)]
        value: String,
        device: Option<String>,
    }
    let pdata = parse_posted_data!(req, res, PostedData);
    let cuser = current_user!(depot, res);
    let mut conn = db::connect()?;
    let exist_token = get_record_by_param!(req, res, AccessToken, access_tokens, &mut conn);
    if exist_token.user_id != cuser.id {
        return context::render_parse_param_error_json_with_detail(res, "access token is not correct");
    }
    if pdata.name.is_empty() {
        return context::render_parse_param_error_json_with_detail(res, "access token's name is not provide");
    }
    let token = conn.transaction::<AccessToken, crate::Error, _>(|conn| {
        let query = access_tokens::table
            .filter(access_tokens::user_id.eq(cuser.id))
            .filter(access_tokens::id.ne(exist_token.id))
            .filter(access_tokens::name.eq(&pdata.name));
        if diesel_exists!(query, conn) {
            return Err(StatusError::conflict()
                .with_summary("token conflict")
                .with_detail("this name is already taken, please try another.")
                .into());
        }
        let token = diesel::update(&exist_token)
            .set((
                access_tokens::name.eq(&pdata.name),
                access_tokens::updated_by.eq(cuser.id),
                access_tokens::updated_at.eq(Utc::now()),
            ))
            .get_result::<AccessToken>(conn)?;
        Ok(token)
    })?;
    res.render(Json(token));
    Ok(())
}
