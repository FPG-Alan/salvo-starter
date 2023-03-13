use chrono::{Duration, Utc};
use diesel::prelude::*;
use salvo::http::StatusCode;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db::{self, lower};
use crate::models::*;
use crate::schema::*;
use crate::utils::{password, validator};
use crate::{context, AppResult, get_email_domain};
pub mod access_token;
pub mod notification;

pub fn authed_root(path: impl Into<String>) -> Router {
    Router::with_path(path)
        .patch(update)
        .push(
            Router::with_path("update_ident_name")
                .post(update_ident_name)
                .patch(update_ident_name),
        )
        .push(
            Router::with_path("update_password")
                .post(update_password)
                .patch(update_password),
        )
        .push(
            Router::with_path("access_tokens").get(access_token::list).push(
                Router::with_path(r"<id:/\d+/>")
                    .patch(access_token::update)
                    .delete(access_token::delete),
            ),
        )
        .push(
            Router::with_path("notifications")
                .get(notification::list)
                .delete(notification::bulk_delete)
                .push(Router::with_path("mark_all_read").post(notification::mark_all_read))
                .push(Router::with_path("mark_read").post(notification::mark_read))
                .push(
                    Router::with_path(r"<id:/\d+/>")
                        .get(notification::show)
                        .delete(notification::delete),
                ),
        )
}

pub fn public_root(path: impl Into<String>) -> Router {
    Router::with_path(path)
        .push(Router::with_path("resend_verification_email").post(resend_verification_email))
        .push(Router::with_path("complete_registration").post(complete_registration))
        .push(Router::with_path("find").post(find))
        .push(Router::with_path("verify").post(verify))
        .push(Router::with_path("create").post(create))
        .push(Router::with_path("send_security_code").post(send_security_code))
        .push(Router::with_path("test_security_code").post(test_security_code))
        .push(Router::with_path("reset_password").post(reset_password))
}
#[handler]
pub async fn find(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        #[serde(default)]
        user: String,
        #[serde(default)]
        ident_name: String,
        #[serde(default)]
        email: String,
    }
    let mut pdata = parse_posted_data!(req, res, PostedData);
    if pdata.ident_name.is_empty() && pdata.email.is_empty() {
        return context::render_parse_param_error_json(res);
    }
    if !pdata.user.is_empty() {
        if let Ok(()) = validator::validate_email(&pdata.user) {
            pdata.email = pdata.user;
        } else if let Ok(()) = validator::validate_ident_name(&pdata.user) {
            pdata.ident_name = pdata.user;
        }
    }
    let mut conn = db::connect()?;
    let email = if !pdata.email.is_empty() {
        emails::table
            .filter(lower(emails::value).eq(pdata.email.to_lowercase()))
            .first::<Email>(&mut conn)
            .ok()
    } else {
        None
    };
    let user_id = if !pdata.ident_name.is_empty() {
        users::table
            .filter(lower(users::ident_name).eq(pdata.ident_name.to_lowercase()))
            .select(users::id)
            .first::<i64>(&mut conn)
            .unwrap_or_default()
    } else if let Some(email) = &email {
        if !email.is_verified {
            return context::render_status_json(
                res,
                StatusCode::BAD_REQUEST,
                "pending_verified",
                "email is not verified",
                "email is not verified",
            );
        }
        email.user_id
    } else {
        0
    };
    if user_id <= 0 {
        return context::render_not_found_json(res);
    }

    #[derive(Serialize, Debug)]
    struct MaskedEmail {
        id: i64,
        value: String,
    }
    #[derive(Serialize, Debug)]
    struct ResultData {
        user_id: i64,
        email: Option<MaskedEmail>,
    }

    let user = users::table.find(user_id).get_result::<User>(&mut conn)?;
    if !user.is_verified {
        context::render_status_json(
            res,
            StatusCode::BAD_REQUEST,
            "pending_verified",
            "user is not verified",
            "user is not verified",
        )
    } else if  user.is_disabled {
        context::render_status_json(
            res,
            StatusCode::BAD_REQUEST,
            "locked_or_disabled",
            "user locked or disabled",
            "this user is locked or disabled",
        )
    } else {
        let email = email.map(|email| MaskedEmail {
            id: email.id,
            value: crate::mask_email(email.value),
        });
       
        res.render(Json(ResultData { user_id, email }));
        Ok(())
    }
}

#[handler]
pub async fn complete_registration(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        #[serde(default)]
        ident_name: String,
        #[serde(default)]
        display_name: String,
        #[serde(default)]
        password: String,
        #[serde(default)]
        email: String,
    }
    let user_id: i64 = req.query("user_id").unwrap_or(0);
    if user_id <= 0 {
        return context::render_parse_query_error_json_with_detail(res, "user_id is not provide or error format");
    }
    let code_value: String = req.query("security_code").unwrap_or_default();
    if code_value.is_empty() {
        return context::render_parse_query_error_json_with_detail(res, "security_code is not provide or empty");
    }
    let pdata = parse_posted_data!(req, res, PostedData);
    let mut conn = db::connect()?;
    let code = security_codes::table
        .filter(security_codes::user_id.eq(user_id))
        .filter(security_codes::value.eq(code_value))
        .first::<SecurityCode>(&mut conn);
    if code.is_err() {
        return context::render_not_found_json_with_detail(res, "your verification code is not found");
    }
    let code = code.unwrap();
    if code.consumed_at.is_some() {
        return context::render_status_json(
            res,
            StatusCode::BAD_REQUEST,
            "code_consumed",
            "code consumed",
            "your verification code has been consumed",
        );
    }
    if code.expired_at < Utc::now() {
        return context::render_status_json(
            res,
            StatusCode::BAD_REQUEST,
            "code_expired",
            "code expired",
            "your verification code is expired",
        );
    }
    if let Err(msg) = validator::validate_password(&pdata.password) {
        return context::render_parse_data_error_json_with_detail(res, msg);
    }
    let pwd = password::hash(&pdata.password);
    if pwd.is_err() {
        return context::render_internal_server_error_json_with_detail(res, "password hash has error");
    }
    let pwd = pwd.unwrap();
    if pdata.ident_name.is_empty() {
        return context::render_parse_data_error_json_with_detail(res, "username is empty");
    }
    if let Err(msg) = validator::validate_ident_name(&pdata.ident_name) {
        return context::render_parse_data_error_json_with_detail(res, msg);
    }
    if let Err(msg) = validator::validate_generic_name(&pdata.display_name) {
        return context::render_parse_data_error_json_with_detail(res, msg);
    }
    if !diesel_exists!(users::table.find(user_id), &mut conn) {
        return context::render_not_found_json_with_detail(res, "user is not exist");
    }
    let user = conn.transaction::<User, crate::Error, _>(|conn| {
        check_ident_name_other_taken!(Some(user_id), &pdata.ident_name, conn);
        let user = diesel::update(users::table.find(user_id))
            .set((
                users::ident_name.eq(&pdata.ident_name),
                users::display_name.eq(&pdata.display_name),
                users::password.eq(&pwd),
                users::updated_by.eq(user_id),
                users::updated_at.eq(Utc::now()),
            ))
            .get_result::<User>(conn)?;

        diesel::update(
            emails::table
                .filter(emails::user_id.eq(user_id))
                .filter(lower(emails::value).eq(pdata.email.to_lowercase())),
        )
        .set((
            emails::is_verified.eq(true),
            emails::updated_by.eq(user_id),
            emails::updated_at.eq(Utc::now()),
        ))
        .execute(conn)?;

        diesel::delete(security_codes::table.find(code.id)).execute(conn)?;
        Ok(user)
    })?;

    res.render(Json(user));
    Ok(())
}

#[handler]
pub async fn send_security_code(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        user_id: i64,
        email_id: Option<i64>,
    }
    let pdata = parse_posted_data!(req, res, PostedData);
    let mut conn = db::connect()?;
    let cuser = users::table.find(pdata.user_id).get_result::<User>(&mut conn)?;
    if let Some(email_id) = pdata.email_id {
        let email = emails::table
            .filter(emails::id.eq(email_id))
            .first::<Email>(&mut conn)?;
        drop(conn);
        cuser.send_security_code_email(&email.value).await?;
        context::render_done_json_with_detail(
            res,
            format!("verification code sent to {}", crate::mask_email(&email.value)),
        )
    } else {
        context::render_parse_data_error_json_with_detail(res, "posted data is invalid")
    }
}

#[handler]
pub async fn test_security_code(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        #[serde(default)]
        user_id: i64,
        #[serde(default)]
        security_code: String,
    }
    #[derive(Serialize, Debug)]
    struct ResultData<'a> {
        is_valid: bool,
        message: &'a str,
    }
    let pdata = parse_posted_data!(req, res, PostedData);
    if pdata.user_id <= 0 || pdata.security_code.is_empty() {
        return context::render_parse_data_error_json(res);
    }
    let mut conn = db::connect()?;
    let code = security_codes::table
        .filter(security_codes::user_id.eq(pdata.user_id))
        .filter(security_codes::value.eq(&pdata.security_code))
        .first::<SecurityCode>(&mut conn)
        .ok();
    if code.is_none() {
        return context::render_not_found_json_with_detail(
            res,
            "You have entered an invalid code. Please check your email and try again. ",
        );
    }
    let code = code.unwrap();

    if code.expired_at < Utc::now() {
        return context::render_parse_data_error_json_with_detail(res, "Your verification code has expired. ");
    }

    match code.consumed_at {
        Some(_) => {
            res.render(Json(ResultData {
                is_valid: false,
                message: "This verification code has already been used.",
            }));
        }
        None => {
            res.render(Json(ResultData {
                is_valid: true,
                message: "This verification code is valid",
            }));
        }
    }
    Ok(())
}

#[handler]
pub async fn reset_password(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        #[serde(default)]
        user_id: i64,
        #[serde(default)]
        security_code: String,
        #[serde(default)]
        password: String,
    }
    let pdata = parse_posted_data!(req, res, PostedData);
    if pdata.security_code.is_empty() {
        return context::render_parse_data_error_json_with_detail(res, "verification code is not provide or empty");
    }
    if let Err(msg) = validator::validate_password(&pdata.password) {
        return context::render_parse_data_error_json_with_detail(res, &msg);
    }
    let mut conn = db::connect()?;
    let code = security_codes::table
        .filter(security_codes::user_id.eq(pdata.user_id))
        .filter(security_codes::value.eq(&pdata.security_code))
        .first::<SecurityCode>(&mut conn);
    if code.is_err() {
        return context::render_parse_data_error_json_with_detail(
            res,
            "You have entered an invalid code. Please check your email and try again. ",
        );
    }
    let code = code.unwrap();
    if code.user_id != pdata.user_id {
        return context::render_parse_data_error_json_with_detail(
            res,
            "You have entered an invalid code. Please check your email and try again. ",
        );
    }
    if code.consumed_at.is_some() {
        return context::render_parse_data_error_json_with_detail(res, "This verification code has already been used.");
    }
    if code.expired_at < Utc::now() {
        return context::render_parse_data_error_json_with_detail(res, "Your verification code has expired. ");
    }
    let user = users::table.find(code.user_id).get_result::<User>(&mut conn)?;
    if  user.is_disabled {
        return context::render_status_json(
            res,
            StatusCode::BAD_REQUEST,
            "locked_or_disabled",
            "user locked or disabled",
            "this user is locked or disabled",
        );
    }

    conn.transaction::<_, crate::Error, _>(|conn| {
        diesel::update(&code)
            .set((
                security_codes::consumed_at.eq(Utc::now()),
                security_codes::updated_at.eq(Utc::now()),
            ))
            .execute(conn)?;
        match password::hash(&pdata.password) {
            Ok(hashed_pwd) => {
                diesel::update(users::table.filter(users::id.eq(code.user_id)))
                    .set((
                        users::password.eq(hashed_pwd),
                        users::updated_by.eq(code.user_id),
                        users::updated_at.eq(Utc::now()),
                    ))
                    .execute(conn)?;
                diesel::delete(access_tokens::table.filter(access_tokens::user_id.eq(user.id))).execute(conn)?;
                Ok(())
            }
            Err(_) => Err(StatusError::internal_server_error().into()),
        }
    })?;
    context::render_done_json_with_detail(res, "password changed")
}

///
/// when user register and did not get verification email, allow user to call this method to resend verification email to the email address he/she used when registered.
/// this method is registered to public router, because user is not login, for only allow user to resend verification email
/// to himself, password is required.
///
#[handler]
pub async fn resend_verification_email(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        user_id: i64,
        password: String,
    }
    let pdata = parse_posted_data!(req, res, PostedData);
    let mut conn = db::connect()?;
    let user = users::table.find(pdata.user_id).first::<User>(&mut conn)?;
    if !password::compare(&pdata.password, &user.password) {
        return context::render_bad_request_json_with_detail(res, "Incorrect username/email or password.");
    }
    if user.is_verified {
        return context::render_not_found_json_with_detail(res, "user is verified already");
    }

    let email = emails::table
        .filter(emails::user_id.eq(pdata.user_id))
        .first::<Email>(&mut conn)?;

    drop(conn);
    user.send_verification_email(&email.value).await?;
    context::render_done_json_with_detail(res, "verification email sent")
}

#[handler]
pub async fn verify(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        #[serde(default)]
        user_id: i64,
        #[serde(default)]
        email: String,
        #[serde(default)]
        token: String,
    }
    let pdata = parse_posted_data!(req, res, PostedData);
    let mut conn = db::connect()?;
    let query = security_codes::user_id
        .eq(pdata.user_id)
        .and(security_codes::value.eq(&pdata.token));
    if pdata.email.is_empty() {
        return context::render_parse_data_error_json_with_detail(res, "email is not provide");
    }
    let code = security_codes::table
        .filter(query.and(security_codes::email.eq(&pdata.email)))
        .first::<SecurityCode>(&mut conn);
    if code.is_err() {
        return context::render_parse_data_error_json_with_detail(res, "your verification code is not exist");
    }
    let code = code.unwrap();
    if code.consumed_at.is_some() {
        return context::render_status_json(
            res,
            StatusCode::BAD_REQUEST,
            "code_consumed",
            "code consumed",
            "your verification code has been consumed",
        );
    }
    if code.expired_at < Utc::now() {
        return context::render_status_json(
            res,
            StatusCode::BAD_REQUEST,
            "code_expired",
            "code expired",
            "your verification code is expired",
        );
    }
    diesel::update(&code)
        .set((
            security_codes::consumed_at.eq(Utc::now()),
            security_codes::updated_by.eq(pdata.user_id),
            security_codes::updated_at.eq(Utc::now()),
        ))
        .execute(&mut conn)?;

    #[derive(Serialize, Debug)]
    struct ResponsedData {
        user: Option<User>,
        email: Option<Email>,
        token: Option<String>,
    }
    let mut data = ResponsedData {
        user: None,
        email: None,
        token: None,
    };
    let mut user = users::table.find(pdata.user_id).get_result::<User>(&mut conn)?;
    // let will_send_welcome = !user.is_verified;
    if !pdata.email.is_empty() {
        let email = diesel::update(
            emails::table
                .filter(emails::user_id.eq(pdata.user_id))
                .filter(lower(emails::value).eq(pdata.email.to_lowercase())),
        )
        .set((
            emails::is_verified.eq(true),
            emails::updated_by.eq(pdata.user_id),
            emails::updated_at.eq(Utc::now()),
        ))
        .get_result::<Email>(&mut conn)?;
        
        if !user.is_verified {
            user = diesel::update(&user)
                .set(users::is_verified.eq(true))
                .get_result::<User>(&mut conn)?;
        }
        match super::auth::create_token(&user, &mut conn) {
            Ok(jwt_token) => {
                res.add_cookie(super::auth::create_token_cookie(jwt_token.clone()));
                data.token = Some(jwt_token);
            }
            Err(msg) => {
                return context::render_invalid_data_json_with_detail(res, &msg);
            }
        }
        data.user = Some(user);
        data.email = Some(email);
    }
    drop(conn);
    res.render(Json(data));
    Ok(())
}

#[handler]
pub async fn create(req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        ident_name: String,
        display_name: String,
        password: String,
        #[serde(default)]
        email: PostedEmail,
    }
    let pdata = parse_posted_data!(req, res, PostedData);
    if !pdata.ident_name.is_empty() {
        if let Err(msg) = validator::validate_ident_name(&pdata.ident_name) {
            return context::render_invalid_data_json_with_detail(res, &msg);
        }
    }
    if let Err(msg) = validator::validate_generic_name(&pdata.display_name) {
        return context::render_invalid_data_json_with_detail(res, &msg);
    }
    if let Err(msg) = validator::validate_email(&pdata.email.value) {
        return context::render_invalid_data_json_with_detail(res, &msg);
    }
    if let Err(msg) = validator::validate_password(&pdata.password) {
        return context::render_parse_data_error_json_with_detail(res, msg);
    }

    let pwd = password::hash(&pdata.password);
    if pwd.is_err() {
        return context::render_internal_server_error_json_with_detail(res, "password hash has error");
    }

    let pwd = pwd.unwrap();
    let mut conn = db::connect()?;
    let (user, email) = conn.transaction::<(User, Email), crate::Error, _>(|conn| {
        let ident_name = if pdata.ident_name.is_empty() {
            crate::generate_ident_name(conn)?
        } else {
            check_ident_name_preserved!(&pdata.ident_name);
            check_ident_name_other_taken!(None, &pdata.ident_name, conn);
            pdata.ident_name.clone()
        };
        check_email_other_taken!(None, &pdata.email.value, conn);

        let new_user = NewUser {

            ident_name: &ident_name,
            display_name: &pdata.display_name,
            password: &pwd,
            in_kernel: false,
            is_verified: true,

            updated_by: None,
            created_by: None,
        };
        let new_user = diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<User>(conn)?;


        let new_email = NewEmail {
            user_id: new_user.id,
            value: &pdata.email.value,
            domain: get_email_domain(&pdata.email.value),
            is_verified: false,
            updated_by: None,
            created_by: None,
        };

        let new_email = diesel::insert_into(emails::table)
            .values(&new_email)
            .get_result::<Email>(conn)?;
        Ok((new_user, new_email))
    })?;
    drop(conn);
    // user.send_verification_email(&email.value).await?;
    res.render(Json(user));
    Ok(())
}

#[handler]
pub async fn update_ident_name(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        #[serde(default)]
        ident_name: String,
    }
    let pdata = parse_posted_data!(req, res, PostedData);
    if pdata.ident_name.is_empty() {
        return context::render_parse_data_error_json_with_detail(res, "username is not provide");
    }
    if let Err(msg) = validator::validate_ident_name(&pdata.ident_name) {
        return context::render_parse_data_error_json_with_detail(res, msg);
    }
    let cuser = current_user!(depot, res);
    let mut conn = db::connect()?;
    let cuser = conn.transaction::<User, crate::Error, _>(|conn| {
        check_ident_name_other_taken!(Some(cuser.id), &pdata.ident_name, conn);
        let cuser = diesel::update(users::table.find(cuser.id))
            .set((
                users::ident_name.eq(&pdata.ident_name),
                users::updated_by.eq(cuser.id),
                users::updated_at.eq(Utc::now()),
            ))
            .get_result::<User>(conn)?;
        Ok(cuser)
    })?;
    res.render(Json(cuser));
    Ok(())
}

#[handler]
pub async fn update_password(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(Deserialize, Debug)]
    struct PostedData {
        #[serde(default)]
        current_password: String,
        #[serde(default)]
        password: String,
    }
    let pdata = parse_posted_data!(req, res, PostedData);
    if pdata.password.is_empty() {
        return context::render_parse_data_error_json_with_detail(res, "password is not provide");
    }
    if let Err(msg) = validator::validate_password(&pdata.password) {
        return context::render_parse_data_error_json_with_detail(res, msg);
    }
    let cuser = current_user!(depot, res);
    if pdata.current_password.is_empty() {
        return context::render_parse_data_error_json_with_detail(res, "current password is not provide");
    }
    if !password::compare(&pdata.current_password, &cuser.password) {
        return context::render_parse_data_error_json_with_detail(res, "current password is not correct");
    }
    let pwd = password::hash(&pdata.password);
    if pwd.is_err() {
        return context::render_internal_server_error_json_with_detail(res, "password hash has error");
    }
    let mut conn = db::connect()?;
    diesel::update(cuser)
        .set((
            users::password.eq(pwd.unwrap()),
            users::updated_by.eq(cuser.id),
            users::updated_at.eq(Utc::now()),
        ))
        .execute(&mut conn)?;
    let exp = Utc::now() + Duration::days(14);
    let jwt_token = crate::create_jwt_token(cuser, &exp);
    diesel::delete(access_tokens::table.filter(access_tokens::user_id.eq(cuser.id))).execute(&mut conn)?;
    if jwt_token.is_err() {
        return context::render_internal_server_error_json_with_detail(res, "generate jwt token error");
    }
    let jwt_token = jwt_token.unwrap();
    let new_token = NewAccessToken {
        user_id: cuser.id,
        kind: "web",
        value: &jwt_token,
        device: None,
        name: None,
        expired_at: exp,
        updated_by: Some(cuser.id),
        created_by: Some(cuser.id),
    };
    diesel::insert_into(access_tokens::table)
        .values(&new_token)
        .execute(&mut conn)?;
    #[derive(Serialize, Debug)]
    struct ResultData<'a> {
        jwt_token: &'a str,
    }
    res.render(Json(ResultData { jwt_token: &jwt_token }));
    Ok(())
}

#[handler]
pub async fn update(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    #[derive(AsChangeset, Deserialize, Debug)]
    #[diesel(table_name = users)]
    struct PostedData {
        display_name: Option<String>
    }
    let pdata = parse_posted_data!(req, res, PostedData);
    let cuser = current_user!(depot, res);
    let mut conn = db::connect()?;
    let user = diesel::update(cuser).set(&pdata).get_result::<User>(&mut conn)?;
    res.render(Json(user));
    Ok(())
}
