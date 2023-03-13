use salvo::http::StatusCode;
use salvo::prelude::*;

use crate::models::*;
use crate::{AppResult, ErrorWrap, StatusWrap};

#[inline]
pub fn current_user(depot: &Depot) -> Option<&User> {
    depot.get::<User>("current_user")
}

#[inline]
pub fn render_status_json<N: Into<String>, S: Into<String>, D: Into<String>>(
    res: &mut Response,
    http_code: StatusCode,
    name: N,
    summary: S,
    detail: D,
) -> AppResult<()> {
    res.set_status_code(http_code);
    if http_code.is_client_error() || http_code.is_server_error() {
        res.render(Json(ErrorWrap::new(http_code, name, summary, detail)));
    } else {
        res.render(Json(StatusWrap::new(http_code, name, summary, detail)));
    }
    Ok(())
}

macro_rules! render_statuses {
    ($($fname: ident, $fdname: ident, $code: expr, $name: expr, $summary: expr, $detail: expr);+) => {
        $(
            #[inline]
            // #[allow(dead_code)]
            pub fn $fdname<D: Into<String>>(res: &mut ::salvo::http::Response, detail: D) -> AppResult<()> {
                render_status_json(res, $code, $name, $summary, detail)
            }
            #[inline]
            // #[allow(dead_code)]
            pub fn $fname(res: &mut ::salvo::http::Response) -> AppResult<()> {
                render_status_json(res, $code, $name, $summary, $detail)
            }
        )+
    }
}

render_statuses! {
    render_parse_param_error_json, render_parse_param_error_json_with_detail, StatusCode::BAD_REQUEST, "parse_param_error", "parse param error", "error happened when parse url param";
    render_parse_query_error_json, render_parse_query_error_json_with_detail, StatusCode::BAD_REQUEST, "parse_query_error", "parse query error", "error happened when parse http query";
    render_parse_data_error_json, render_parse_data_error_json_with_detail, StatusCode::BAD_REQUEST, "parse_data_error", "parse data error", "error happened when parse posted data";
    render_internal_server_error_json, render_internal_server_error_json_with_detail, StatusCode::INTERNAL_SERVER_ERROR, "internal_server_error", "internal server error", "internal server error happened";
    render_conflict_error_json, render_conflict_error_json_with_detail, StatusCode::BAD_REQUEST, "conflict_error", "conflict error", "conflict error happened";
    render_bad_request_json, render_bad_request_json_with_detail, StatusCode::BAD_REQUEST, "bad_request_error", "bad request error", "bad request error";
    render_db_error_json, render_db_error_json_with_detail, StatusCode::INTERNAL_SERVER_ERROR, "db_error", "db error", "unkown db error happened";
    render_not_found_json, render_not_found_json_with_detail, StatusCode::NOT_FOUND, "not_found", "not found error", "this resource is not found or access denied";
    render_invalid_data_json, render_invalid_data_json_with_detail, StatusCode::BAD_REQUEST, "invalid_data", "invalid data", "data is an invalid";
    render_invalid_user_json, render_invalid_user_json_with_detail, StatusCode::BAD_REQUEST, "invalid_user", "invalid user", "current user is an invalid user";
    render_access_denied_json, render_access_denied_json_with_detail, StatusCode::FORBIDDEN, "access_denied", "access denied", "no permission to access this record";
    render_done_json, render_done_json_with_detail, StatusCode::OK, "done", "done", "done"
}
pub async fn parse_ids_from_request(req: &mut Request, sg_name: &str, pl_name: &str) -> Vec<i64> {
    if let Some(idstrs) = req.form_or_query::<String>(pl_name).await {
        let mut ids = vec![];
        for idstr in idstrs.split(',') {
            if let Ok(id) = idstr.parse::<i64>() {
                ids.push(id);
            }
        }
        ids
    } else if let Some(id) = req.form_or_query::<i64>(sg_name).await {
        vec![id]
    } else {
        #[derive(Deserialize, Debug)]
        struct PostedData {
            #[serde(default)]
            id: Option<i64>,
            #[serde(default)]
            ids: Option<Vec<i64>>,
        }
        if let Ok(pdata) = req.parse_json::<PostedData>().await {
            if let Some(pids) = pdata.ids {
                pids
            } else if let Some(id) = pdata.id {
                vec![id]
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }
}
