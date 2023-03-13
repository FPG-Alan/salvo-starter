use serde::Serialize;
use diesel::prelude::*;

#[derive(Serialize, Debug)]
pub struct PagedData<T> {
    pub records: Vec<T>,
    pub limit: i64,
    pub offset: i64,
    pub total: i64,
    pub sort: Option<String>,
}

