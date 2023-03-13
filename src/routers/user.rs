use chrono::Utc;
use diesel::prelude::*;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db;
use crate::models::*;
use crate::schema::*;
use crate::utils::{validator};
use crate::{context, AppResult};

pub fn authed_root(path: impl Into<String>) -> Router {
    Router::with_path(path)
        .get(list)
        .delete(bulk_delete)
        .push(
            Router::with_path(r"<id:/\d+/>")
                .get(show)
                .patch(update)
                .delete(delete)
                .push(Router::with_path("set_disabled").post(set_disabled))
                .push(Router::with_path("emails").get(list_emails))
        )
}

pub fn public_root(path: impl Into<String>) -> Router {
    Router::with_path(path).push(Router::with_path("is_other_taken").handle(is_other_taken))
}

#[handler]
pub async fn show(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let mut conn = db::connect()?;
    show_record!(req, depot, res, User, users, &mut conn);
    Ok(())
}
#[handler]
pub async fn list(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let cuser = current_user!(depot, res);
    let mut conn = db::connect()?;
    let query = users::table.filter(users::is_disabled.eq(false));
    list_records!(
        req,
        res,
        User,
        query,
        "updated_at desc",
        USER_FILTER_FIELDS.clone(),
        USER_JOINED_OPTIONS.clone(),
        USER_SEARCH_TMPL,
        &mut conn
    );
    Ok(())
}

#[handler]
pub async fn delete(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let mut conn = db::connect()?;
    delete_record!(req, depot, res, users, User, db::delete_user, &mut conn);
    Ok(())
}
#[handler]
pub async fn bulk_delete(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let mut conn = db::connect()?;
    bulk_delete_records!(req, depot, res, users, User, db::delete_user, &mut conn);
    Ok(())
}


#[handler]
pub async fn list_emails(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let cuser = current_user!(depot, res);
    let mut conn = db::connect()?;
    let user = get_record_by_param!(req, res, User, users, &mut conn);

    let uemails = emails::table
        .filter(emails::user_id.eq(user.id))
        .get_results::<Email>(&mut conn)?;
    res.render(Json(uemails));
    Ok(())
}

#[handler]
pub async fn update(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(AsChangeset, Deserialize, Debug)]
    #[diesel(table_name = users)]
    struct PostedData {
        display_name: Option<String>,
    }
    let pdata = parse_posted_data!(req, res, PostedData);
    let cuser = current_user!(depot, res);
    let mut conn = db::connect()?;
    let user = get_record_by_param!(req, res, User, users, &mut conn);

    let user = diesel::update(&user).set(&pdata).get_result::<User>(&mut conn)?;
    res.render(Json(user));
    Ok(())
}

#[handler]
pub async fn set_disabled(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        value: bool,
    }
    let pdata = parse_posted_data!(req, res, PostedData);
    let cuser = current_user!(depot, res);
    let mut conn = db::connect()?;
    let user = get_record_by_param!(req, res, User, users, &mut conn);
    if user.id == cuser.id  || !cuser.in_kernel {
        return context::render_access_denied_json(res);
    }
    let user = if pdata.value {
        diesel::update(&user)
            .set((users::is_disabled.eq(pdata.value), users::disabled_at.eq(Utc::now())))
            .get_result::<User>(&mut conn)?
    } else {
        diesel::update(&user)
            .set(users::is_disabled.eq(pdata.value))
            .get_result::<User>(&mut conn)?
    };
    res.render(Json(user));
    Ok(())
}

#[handler]
pub async fn is_other_taken(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let user_id = req.query::<i64>("user_id");
    let ident_name: String = req.query("ident_name").unwrap_or_default();
    let email_value: String = req.query("email").unwrap_or_default();
    let mut taken = false;
    let mut conn = db::connect()?;
    if !ident_name.is_empty() {
        taken = validator::is_ident_name_other_taken(user_id, &ident_name, &mut conn)?;
    }
    if !taken && !email_value.is_empty() {
        taken = validator::is_email_other_taken(user_id, &email_value, &mut conn)?;
    }
    #[derive(Serialize, Debug)]
    struct ResultData {
        taken: bool,
    }
    res.render(Json(ResultData { taken }));
    Ok(())
}