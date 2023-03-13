use chrono::Utc;
use diesel::prelude::*;
use salvo::prelude::*;

use crate::models::*;
use crate::schema::*;
use crate::{context, db, AppResult};

#[handler]
pub async fn show(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let mut conn = db::connect()?;
    show_record!(req, depot, res, Notification, notifications, &mut conn);
    Ok(())
}
#[handler]
pub async fn delete(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let mut conn = db::connect()?;
    delete_record!(
        req,
        depot,
        res,
        notifications,
        Notification,
        db::delete_notification,
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
        notifications,
        Notification,
        db::delete_notification,
        &mut conn
    );
    Ok(())
}

#[handler]
pub async fn list(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let cuser = current_user!(depot, res);
    let query = notifications::table.filter(notifications::owner_id.eq(cuser.id));
    let mut conn = db::connect()?;
    list_records!(
        req,
        res,
        Notification,
        query,
        "updated_at desc",
        NOTIFICATION_FILTER_FIELDS.clone(),
        NOTIFICATION_JOINED_OPTIONS.clone(),
        ID_NAME_SEARCH_TMPL,
        &mut conn
    );
    Ok(())
}
#[handler]
pub async fn mark_read(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let cuser = current_user!(depot, res);
    let notification_id: i64 = req.query("id").or_else(|| req.query("notification_id")).unwrap_or(0);
    let mut conn = db::connect()?;
    if notification_id > 0 {
        diesel::update(
            notifications::table
                .filter(notifications::id.eq(notification_id))
                .filter(notifications::owner_id.eq(cuser.id)),
        )
        .set((
            notifications::is_read.eq(true),
            notifications::updated_by.eq(cuser.id),
            notifications::updated_at.eq(Utc::now()),
        ))
        .execute(&mut conn)?;
    }
    
    context::render_done_json(res)
}
#[handler]
pub async fn mark_all_read(_req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let cuser = current_user!(depot, res);
    let mut conn = db::connect()?;
    diesel::update(notifications::table.filter(notifications::owner_id.eq(cuser.id)))
        .filter(notifications::is_read.eq(false))
        .set((
            notifications::is_read.eq(true),
            notifications::updated_by.eq(cuser.id),
            notifications::updated_at.eq(Utc::now()),
        ))
        .execute(&mut conn)?;
    context::render_done_json(res)
}
