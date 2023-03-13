use diesel::pg::PgConnection;
use diesel::prelude::*;
use crate::schema::*;
use crate::{AppResult};

pub fn delete_user(id: i64, conn: &mut PgConnection) -> AppResult<()> {
    conn.transaction::<_, crate::Error, _>(|conn| {
        diesel::delete(security_codes::table.filter(security_codes::user_id.eq(id))).execute(conn)?;
        diesel::delete(emails::table.filter(emails::user_id.eq(id))).execute(conn)?;
        diesel::delete(users::table.find(id)).execute(conn)?;
        Ok(())
    })
}

pub fn delete_access_token(id: i64, conn: &mut PgConnection) -> Result<(), diesel::result::Error> {
    diesel::delete(access_tokens::table.filter(access_tokens::id.eq(id))).execute(conn)?;
    Ok(())
}
pub fn delete_notification(id: i64, conn: &mut PgConnection) -> Result<(), diesel::result::Error> {
    diesel::delete(notifications::table.filter(notifications::id.eq(id))).execute(conn)?;
    Ok(())
}
