#[macro_export]
macro_rules! get_id_param {
    ($req:expr, $res:expr, $name:expr) => {{
        let value = $req.param($name).unwrap_or(0i64);
        if value <= 0 {
            return $crate::context::render_parse_param_error_json($res);
        }
        value
    }};
    ($req:expr, $res:expr) => {
        get_id_param!($req, $res, "id")
    };
}
#[macro_export]
macro_rules! get_id_query {
    ($req:expr, $res:expr, $name:expr) => {{
        let value = $req.query($name).unwrap_or(0i64);
        if value <= 0 {
            return $crate::context::render_parse_param_error_json($res);
        }
        value
    }};
}

#[macro_export]
macro_rules! get_record {
    ($res:expr, $id:expr, $model:ty, $edb:path, $conn:expr) => {
        {
            use $edb as edb; //https://github.com/rust-lang/rust/issues/48067
            match edb::table.find($id).first::<$model>($conn) {
                Ok(record) => {
                    record
                },
                Err(diesel::result::Error::NotFound) => {
                    // tracing::info!(sql = %debug_query!(&edb::table.find($id)), "diesel get_record not found");
                    return $crate::context::render_not_found_json($res);
                },
                Err(e) => {
                    tracing::error!(error = ?e, sql = %debug_query!(&edb::table.find($id)), "get record by id error");
                    return $crate::context::render_db_error_json($res)
                },
            }
        }
    };
}

#[macro_export]
macro_rules! get_permitted_record {
    ($user:expr, $action:expr, $res:expr, $id:expr, $model:ty, $edb:path, $conn:expr) => {{
        use $edb as edb; //https://github.com/rust-lang/rust/issues/48067
        edb::table
            .filter(edb::id.eq($id))
            .get_result::<$model>($conn)?
    }};
}
#[macro_export]
macro_rules! get_permitted_record_by_param {
    ($user:expr, $action:expr, $req:expr, $res:expr, $model:ty, $edb:path, $param:expr, $conn:expr) => {{
        use diesel::prelude::*;
        let id = get_id_param!($req, $res, $param);
        get_permitted_record!($user, $action, $res, id, $model, $edb, $conn)
    }};
    ($user:expr, $action:expr, $req:expr, $res:expr, $model:ty, $edb:path, $conn:expr) => {
        get_permitted_record_by_param!($user, $action, $req, $res, $model, $edb, "id", $conn)
    };
}
#[macro_export]
macro_rules! get_permitted_record_by_query {
    ($req:expr, $res:expr, $model:ty, $edb:path, $query:expr, $conn:expr) => {{
        use diesel::prelude::*;
        use $edb as edb;
        let id = get_id_query!($req, $res, $query);
        get_permitted_record!($res, id, $conn, $edb, $model)
    }};
}

#[macro_export]
macro_rules! get_record_by_param {
    ($req:expr, $res:expr, $model:ty, $edb:path, $param:expr, $conn:expr) => {{
        use diesel::prelude::*;
        let id = get_id_param!($req, $res, $param);
        get_record!($res, id, $model, $edb, $conn)
    }};
    ($req:expr, $res:expr, $model:ty, $edb:path, $conn:expr) => {
        get_record_by_param!($req, $res, $model, $edb, "id", $conn)
    };
}
#[macro_export]
macro_rules! get_record_by_query {
    ($req:expr, $res:expr, $model:ty, $edb:path, $query:expr, $conn:expr) => {{
        use diesel::prelude::*;
        let id = get_id_query!($req, $res, $query);
        get_record!($res, id, $model, $edb, $conn)
    }};
}
#[macro_export]
macro_rules! current_user {
    ($depot:expr, $res:expr) => {{
        let cuser = $crate::context::current_user($depot);
        if cuser.is_none() {
            return $crate::context::render_invalid_user_json($res);
        }
        cuser.unwrap()
    }};
}

#[macro_export]
macro_rules! parse_posted_data {
    ($req:expr, $res:expr, $model:ty) => {{
        match $req.parse_json::<$model>().await {
            Ok(pdata) => pdata,
            Err(e) => {
                tracing::error!(error = ?e, "posted data parse error");
                return $crate::context::render_parse_data_error_json($res);
            }
        }
    }};
}

#[macro_export]
macro_rules! show_record {
    ($req:expr, $depot:expr, $res:expr, $model:ty, $edb:path, $qid:expr, $conn:expr) => {
        // println!("===open db conn in show_record");
        let record = get_record_by_param!($req, $res, $model, $edb, $conn);
        let cuser = current_user!($depot, $res);
        $res.render(Json(record));
    };
    ($req:expr, $depot:expr, $res:expr, $model:ty, $edb:path, $conn:expr) => {
        show_record!($req, $depot, $res, $model, $edb, "id", $conn);
    };
    ($req:expr, $depot:expr, $res:expr, $model:ty, $edb:path, $dep_edb:path, $dep_model:ty, $cfield:ident, $action:expr, $conn:expr) => {
        use $dep_edb as dep_edb;
        // println!("===open db conn in show_record2");
        let record = get_record_by_param!($req, $res, $model, $edb, $conn);
        let cuser = current_user!($depot, $res);
        let dep = dep_edb::table.find(record.$cfield).get_result::<$dep_model>($conn);
        if dep.is_err() {
            return $crate::context::render_not_found_json($res);
        }
        let dep = dep.unwrap();
        $res.render(Json(record));
    };
}

#[macro_export]
macro_rules! list_records {
    ($req:expr, $res:expr, $model:ty, $query:expr, $default_sort:expr, $filter_fields:expr, $joined_options:expr, $search_tmpl:expr, $conn:expr) => {{
        let data = query_pagation_data!(
            $req,
            $res,
            $model,
            $query,
            $default_sort,
            $filter_fields,
            $joined_options,
            $search_tmpl,
            $conn
        );
        $res.render(Json(&data));
        data
    }}; // ($req:expr, $res:expr, $model:ty, $query:expr, $default_sort:expr, $filter_fields:expr, $joined_options:expr, $search_tmpl:expr) => {{
        //     println!("===open db conn in list_record");
        //     let mut conn = $crate::db::connect()?;
        //     list_records!(
        //         $req,
        //         $res,
        //         $model,
        //         $query,
        //         &mut conn,
        //         $default_sort,
        //         $filter_fields,
        //         $joined_options,
        //         $search_tmpl
        //     );
        //     drop(conn);
        // }};
}
#[macro_export]
macro_rules! delete_record {
    ($req:expr, $depot:expr, $res:expr, $edb:path, $model:ty, $del:expr, $conn:expr) => {{
        let cuser = current_user!($depot, $res);
        let record = get_record_by_param!($req, $res, $model, $edb, $conn);
        $del(record.id, $conn)?;
        $crate::context::render_done_json($res).ok();
        record
    }};
    ($req:expr, $depot:expr, $res:expr, $edb:path, $model:ty, $del:expr, $dep_edb:path, $dep_model:ty, $cfield:ident, $action:expr, $conn:expr) => {{
        use $dep_edb as dep_edb;
        let cuser = current_user!($depot, $res);
        let record = get_record_by_param!($req, $res, $model, $edb, $conn);
        let dep = dep_edb::table.find(record.$cfield).get_result::<$dep_model>($conn);
        if dep.is_err() {
            return $crate::context::render_not_found_json($res);
        }
        let dep = dep.unwrap();
        $del(record.id, $conn)?;
        $crate::context::render_done_json($res).ok();
        record
    }};
}
#[macro_export]
macro_rules! bulk_delete_records {
    ($req:expr, $depot:expr, $res:expr, $edb:path, $model:ty, $del:expr, $conn:expr) => {{
        use $edb as edb;
        let ids = $crate::context::parse_ids_from_request($req, "id", "ids").await;
        let cuser = current_user!($depot, $res);

        let records = edb::table.filter(edb::id.eq_any(&ids)).get_results::<$model>($conn)?;
        let mut done_ids = vec![];
        let mut deined_ids = vec![];
        let mut nerr_ids = vec![];
        for record in &records {
            if $del(record.id, $conn).is_err() {
                nerr_ids.push(record.id);
            } else {
                done_ids.push(record.id);
            }
        }
        render_bulk_action_json!(
            $res,
            done_ids,
            (deined_ids, "denied_access", "denied access", "denied access"),
            (nerr_ids, "unknown_error", "unknown error", "unknown error")
        );
        records
    }};
    ($req:expr, $depot:expr, $res:expr, $edb:path, $model:ty, $del:expr, $dep_edb:path, $dep_model:ty, $cfield:tt, $action:expr, $conn:expr) => {{
        use $dep_edb as dep_edb;
        use $edb as edb;
        let ids = $crate::context::parse_ids_from_request($req, "id", "ids").await;
        let cuser = current_user!($depot, $res);

        let records = edb::table.filter(edb::id.eq_any(&ids)).get_results::<$model>($conn);
        if records.is_err() {
            return $crate::context::render_db_error_json($res);
        }
        let records = records.unwrap();
        let mut deps: HashMap<i64, $dep_model> = std::collections::HashMap::new();
        let mut done_ids = vec![];
        let mut deined_ids = vec![];
        let mut nerr_ids = vec![];
        for record in &records {
            if deps.get(&record.$cfield).is_none() {
                let dep = dep_edb::table.find(record.$cfield).get_result::<$dep_model>($conn);
                if dep.is_err() {
                    nerr_ids.push(record.id);
                    continue;
                }
                deps.insert(record.$cfield.clone(), dep.unwrap());
            }
            if $del(record.id, $conn).is_err() {
                nerr_ids.push(record.id);
            } else {
                done_ids.push(record.id);
            }
        }
        render_bulk_action_json!(
            $res,
            done_ids,
            (deined_ids, "denied_access", "denied access", "denied access"),
            (nerr_ids, "unknown_error", "unknown error", "unknown error")
        );
        records
    }};
}
#[macro_export]
macro_rules! url_filter_joined_options {
    ($($outer_table:expr, $inner_key:expr=>$outer_key:expr, $($url_field:expr=>$o_field:expr),+;)*) => {
        {
            let mut options = vec![];
            $(
                let mut map = std::collections::HashMap::new();
                $(
                    map.insert($url_field.into(), $o_field.into());
                )+
                options.push($crate::db::url_filter::JoinedOption {
                    outer_table: $outer_table.into(),
                    outer_key: $outer_key.into(),
                    inner_key: $inner_key.into(),
                    url_name_map: map,
                });
            )*
            options
        }
    };
}

#[macro_export]
macro_rules! query_pagation_data {
    ($req:expr, $res:expr, $model:ty, $query:expr, $default_sort:expr, $filter_fields:expr, $joined_options:expr, $search_tmpl:expr, $conn:expr) => {
        {
            use diesel::prelude::*;
            use $crate::db::Paginate;

            let offset = $req.query::<i64>("offset").map(|l|{
                if l < 0 {
                    0
                } else {
                    l
                }
            }).unwrap_or(0);
            let limit = $req.query::<i64>("limit").map(|l|{
                if l > 200 || l <= 0 {
                    200
                } else {
                    l
                }
            }).unwrap_or(200);

            let sort = $req.query::<String>("sort");
            let sort = match &sort {
                Some(sort) => {
                    if &*sort == "" || $crate::utils::validator::validate_db_sort(sort).is_err() {
                        $default_sort
                    } else {
                        sort
                    }
                },
                None => $default_sort,
            };
            let filter = $req.query::<String>("filter").unwrap_or_default();
            let mut search = $req.query::<String>("search").unwrap_or_default();
            search.retain(|c|c != '\'' && c != '\"');
            let search =search.replace("_", "\\_");
            let filter = if search.is_empty() || $search_tmpl.is_empty() {
                filter
            } else {
                let hb = handlebars::Handlebars::new();
                let mut data = std::collections::HashMap::new();
                data.insert("data", &search);
                match hb.render_template($search_tmpl, &data) {
                    Ok(search) => if !filter.is_empty() {
                        format!("({}) and ({})", &filter, &search)
                    } else {
                        search
                    },
                    Err(e) => {
                        tracing::error!(error = ?e, tmpl = %$search_tmpl, data = %search, "search template error");
                        filter
                    }
                }
            };
            let mut parser = $crate::db::url_filter::Parser::new(filter, $filter_fields, $joined_options);
            let filter = match parser.parse() {
                Ok(filter) => filter,
                Err(msg) => {
                    tracing::info!( error = %msg, "parse url filter error");
                    "".into()
                },
            };
            // tracing::info!( filter = %filter, "url data filter");

            let (records, total) = if !filter.is_empty() {
                let query = $query.filter(::diesel::dsl::sql::<::diesel::sql_types::Bool>(&format!("({})", filter))).order(::diesel::dsl::sql::<::diesel::sql_types::Text>(sort)).paginate(offset).limit(limit);
                // print_query!(&query);
                query.load_and_total::<$model>($conn)?
            } else {
                let query = $query.order(::diesel::dsl::sql::<::diesel::sql_types::Text>(sort)).paginate(offset).limit(limit);
                // print_query!(&query);
                query.load_and_total::<$model>($conn)?
            };
            $crate::data::PagedData{
                records,
                limit,
                offset,
                total,
                sort: Some(sort.to_string()),
            }
        }
    };
}

#[macro_export]
macro_rules! join_path {
    ($($part:expr),+) => {
        {
            let mut p = std::path::PathBuf::new();
            $(
                p.push($part);
            )*
            path_slash::PathBufExt::to_slash_lossy(&p).to_string()
        }
    }
}

#[macro_export]
macro_rules! create_bulk_action_result_data {
    ($done_ids:expr, $(($ids:expr, $name:expr, $summary:expr, $detail:expr)),+) => {
        {
            let mut result = $crate::BulkResultData{done_ids: $done_ids, errors: vec![]};
            $(
                if !$ids.is_empty() {
                    result.errors.push($crate::BulkErrorInfo{
                        record_ids: $ids,
                        name: $name.into(),
                        summary: $summary.into(),
                        detail: $detail.into(),
                    });
                }
            )+
            result
        }
    };

    ($done_ids:expr) => {
        {
            $crate::BulkResultData{done_ids: $done_ids, errors: vec![]}
        }
    };
}

#[macro_export]
macro_rules! check_ident_name_preserved {
    ($name:expr) => {
        if $crate::is_ident_name_preserved($name) {
            return Err(salvo::http::StatusError::conflict()
                .with_summary("name preserved")
                .with_detail("this name is preserved")
                .into());
        }
    };
}

#[macro_export]
macro_rules! check_ident_name_other_taken {
    ($user_id:expr, $ident_name:expr, $conn:expr) => {{
        match $crate::utils::validator::is_ident_name_other_taken($user_id, $ident_name, $conn) {
            Ok(true) => {
                return Err(salvo::http::StatusError::conflict()
                    .with_summary("username conflict")
                    .with_detail("this user name is already taken, please try another.")
                    .into())
            }
            Err(_) => {
                return Err(salvo::http::StatusError::internal_server_error()
                    .with_summary("db error")
                    .with_detail("db error when check username conflict")
                    .into())
            }
            _ => {}
        }
    }};
}
#[macro_export]
macro_rules! check_email_other_taken {
    ($user_id:expr, $email:expr, $conn:expr) => {{
        match $crate::utils::validator::is_email_other_taken($user_id, $email, $conn) {
            Ok(true) => {
                return Err(salvo::http::StatusError::conflict()
                    .with_summary("email conflict")
                    .with_detail("This email is already taken, please try another.")
                    .into())
            }
            Err(_) => {
                return Err(salvo::http::StatusError::internal_server_error()
                    .with_summary("db error")
                    .with_detail("db error when check email conflict")
                    .into())
            }
            _ => {}
        }
    }};
}

#[macro_export]
macro_rules! render_bulk_action_json {
    ($res:expr, $done_ids:expr, $(($ids:expr, $name:expr, $summary:expr, $detail:expr)),+) => {
        {
            $res.render(Json(create_bulk_action_result_data!($done_ids, $(($ids, $name, $summary, $detail)),+)));
        }
    };
    ($res:expr, $done_ids:expr) => {
        {
            $res.render(Json(create_bulk_action_result_data!($done_ids)));
        }
    };
}
#[macro_export]
macro_rules! diesel_exists {
    ($query:expr, $conn:expr) => {{
        // tracing::info!( sql = %debug_query!(&$query), "diesel_exists");
        diesel::select(diesel::dsl::exists($query)).get_result::<bool>($conn)?
    }};
    ($query:expr, $default:expr, $conn:expr) => {{
        // tracing::info!( sql = debug_query!(&$query), "diesel_exists");
        diesel::select(diesel::dsl::exists($query))
            .get_result::<bool>($conn)
            .unwrap_or($default)
    }};
}
#[macro_export]
macro_rules! diesel_exists_result {
    ($query:expr, $conn:expr) => {{
        // tracing::info!( sql = %debug_query!(&$query), "diesel_exists");
        diesel::select(diesel::dsl::exists($query)).get_result::<bool>($conn)
    }};
}

#[macro_export]
macro_rules! print_query {
    ($query:expr) => {
        println!("{}", diesel::debug_query::<diesel::pg::Pg, _>($query));
    };
}

#[macro_export]
macro_rules! debug_query {
    ($query:expr) => {{
        format!("{}", diesel::debug_query::<diesel::pg::Pg, _>($query))
    }};
}
