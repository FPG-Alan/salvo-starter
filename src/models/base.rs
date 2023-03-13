use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::db::url_filter::JoinedOption;
use crate::schema::*;


pub static USER_FILTER_FIELDS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        "id",
        "ident_name",
        "display_name",
        "in_kernel",
    ]
    .into_iter()
    .map(String::from)
    .collect()
});
pub static USER_JOINED_OPTIONS: Lazy<Vec<JoinedOption>> = Lazy::new(|| {
    url_filter_joined_options![
        "emails", "id"=>"user_id", "e.value"=>"value";
    ]
});
pub static USER_SEARCH_TMPL: &str = "id::varchar(255)='{{data}}' or ident_name ilike E'%{{data}}%' or display_name ilike E'%{{data}}%' or e.value ilike E'%{{data}}%' or p.value ilike E'%{{data}}%'";
#[derive(Identifiable, Queryable, Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub id: i64,

    pub ident_name: String,
    pub display_name: String,
    #[serde(skip_serializing)]
    pub password: String,

    pub is_disabled: bool,
    pub disabled_by: Option<i64>,
    pub disabled_at: Option<DateTime<Utc>>,
    

    pub is_verified: bool,
    pub verified_at: Option<DateTime<Utc>>,

    // pub profile: Value,
    // pub avatar: Option<String>,

    pub updated_by: Option<i64>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,

    pub in_kernel: bool,
}
#[derive(Insertable, Deserialize, Clone, Debug)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {

    pub ident_name: &'a str,
    pub display_name: &'a str,
    pub password: &'a str,
    pub in_kernel: bool,
    pub is_verified: bool,
    // pub profile: Value,

    pub updated_by: Option<i64>,
    pub created_by: Option<i64>,
}


pub static EMAIL_FILTER_FIELDS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        "id",
        "user_id",
        "value",
        "is_verified",
        "updated_by",
        "created_by",
    ]
    .into_iter()
    .map(String::from)
    .collect()
});
pub static EMAIL_JOINED_OPTIONS: Lazy<Vec<JoinedOption>> = Lazy::new(Vec::new);
#[derive(Identifiable, Queryable, Serialize, Deserialize, Clone, Debug)]
pub struct Email {
    pub id: i64,
    pub user_id: i64,
    pub value: String,
    pub domain: String,
    pub is_verified: bool,

    pub updated_by: Option<i64>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
}
#[derive(Insertable, Serialize, Clone, Debug)]
#[diesel(table_name = emails)]
pub struct NewEmail<'a> {
    pub user_id: i64,
    pub value: &'a str,
    pub domain: &'a str,
    pub is_verified: bool,

    pub updated_by: Option<i64>,
    pub created_by: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostedEmail {
    #[serde(default)]
    pub value: String,
    #[serde(default = "crate::default_as_false")]
    pub is_subscribed: bool,
}
impl Default for PostedEmail {
    fn default() -> Self {
        PostedEmail {
            value: "".to_owned(),
            is_subscribed: false,
        }
    }
}

pub static ACCESS_TOKEN_FILTER_FIELDS: Lazy<Vec<String>> = Lazy::new(|| {
    vec!["id", "user_id", "name", "kind", "value", "updated_by", "created_by"]
        .into_iter()
        .map(String::from)
        .collect()
});
pub static ACCESS_TOKEN_JOINED_OPTIONS: Lazy<Vec<JoinedOption>> = Lazy::new(Vec::new);
#[derive(Identifiable, Queryable, Serialize, Deserialize, Clone, Debug)]
// #[belongs_to(User)]
pub struct AccessToken {
    pub id: i64,
    pub user_id: i64,
    pub name: Option<String>,
    pub kind: String,
    pub value: String,
    pub device: Option<String>,
    pub expired_at: DateTime<Utc>,

    pub updated_by: Option<i64>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable, Serialize, Clone, Debug)]
#[diesel(table_name = access_tokens)]
pub struct NewAccessToken<'a> {
    pub user_id: i64,
    pub name: Option<&'a str>,
    pub kind: &'a str,
    pub value: &'a str,
    pub device: Option<&'a str>,
    pub expired_at: DateTime<Utc>,

    pub updated_by: Option<i64>,
    pub created_by: Option<i64>,
}


#[derive(Identifiable, Queryable, Serialize, Deserialize, Clone, Debug)]
pub struct SecurityCode {
    pub id: i64,
    pub user_id: i64,
    pub email: Option<String>,
    pub value: String,
    pub send_method: String,
    pub consumed_at: Option<DateTime<Utc>>,
    pub expired_at: DateTime<Utc>,

    pub updated_by: Option<i64>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = security_codes)]
pub struct NewSecurityCode<'a> {
    pub user_id: i64,
    pub email: Option<&'a str>,
    pub value: &'a str,
    pub send_method: &'a str,
    pub expired_at: DateTime<Utc>,

    pub updated_by: Option<i64>,
    pub created_by: Option<i64>,
}


pub static NOTIFICATION_FILTER_FIELDS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        "id",
        "owner_id",
        "sender_id",
        "kind",
        "is_read",
        "updated_by",
        "created_by",
    ]
    .into_iter()
    .map(String::from)
    .collect()
});
pub static NOTIFICATION_JOINED_OPTIONS: Lazy<Vec<JoinedOption>> = Lazy::new(Vec::new);
#[derive(Identifiable, Queryable, Serialize, Clone, Debug)]
#[diesel(table_name = notifications)]
pub struct Notification {
    pub id: i64,
    pub owner_id: i64,
    pub sender_id: Option<i64>,
    pub subject: String,
    pub body: String,
    pub kind: String,
    pub is_read: bool,
    pub extra: Value,

    pub updated_by: Option<i64>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable, Deserialize, Clone, Debug)]
#[diesel(table_name = notifications)]
pub struct NewNotification<'a> {
    pub owner_id: i64,
    pub sender_id: Option<i64>,
    pub subject: &'a str,
    pub body: &'a str,
    pub kind: &'a str,
    pub extra: Value,

    pub updated_by: Option<i64>,
    pub created_by: Option<i64>,
}


#[derive(QueryableByName, Debug)]
pub struct TableId {
    #[diesel(sql_type = ::diesel::sql_types::BigInt)]
    #[diesel(column_name = id)]
    pub id: i64,
}