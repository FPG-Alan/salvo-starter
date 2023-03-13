use diesel::prelude::*;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

use crate::db::lower;
use crate::schema::*;
use crate::AppResult;

pub fn validate_db_sort(sort: &str) -> Result<(), String> {
    if sort.is_empty() {
        return Err("sort is empty".into());
    }
    if sort.len() > 50 {
        return Err("sort is too long".into());
    }
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\w+(\s(asc|desc))?$").unwrap());
    if !RE.is_match(sort) {
        return Err("sort format is invalid".into());
    }
    Ok(())
}

pub fn validate_email<T: AsRef<str>>(email: T) -> Result<(), String> {
    let email = email.as_ref();
    if email.is_empty() {
        return Err("email is empty".into());
    }
    if email.len() > 255 {
        return Err("email is too long".into());
    }
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap());
    if !RE.is_match(email) {
        return Err("email format is invalid".into());
    }
    Ok(())
}


pub fn validate_ident_name<T: AsRef<str>>(username: T) -> Result<(), String> {
    let username = username.as_ref();
    if username.is_empty() {
        return Err("username is empty".into());
    }
    if username.len() > 255 {
        return Err("username is too long".into());
    }
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-z]+[a-z0-9.\-_]{2,}$").unwrap());
    if !RE.is_match(username) {
        return Err("username format is invalid".into());
    }
    Ok(())
}

pub fn validate_password<T: AsRef<str>>(password: T) -> Result<(), String> {
    let password = password.as_ref();
    if password.len() < 8 {
        return Err("password is too short".into());
    }
    if password.len() > 64 {
        return Err("password is too long".into());
    }
    static RE_LC: Lazy<Regex> = Lazy::new(|| Regex::new(r"[a-z]+?").unwrap());
    static RE_UC: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Z]+?").unwrap());
    static RE_DG: Lazy<Regex> = Lazy::new(|| Regex::new(r"\d+?").unwrap());
    if !RE_LC.is_match(password) {
        return Err("password must contains lowercase characters".into());
    }
    if !RE_UC.is_match(password) {
        return Err("password must contains uppercase characters".into());
    }
    if !RE_DG.is_match(password) {
        return Err("password must contains digits".into());
    }
    let mut dict = HashMap::with_capacity(64);
    for c in password.chars() {
        dict.insert(c, true);
    }
    if dict.len() < 4 {
        return Err("password contains at least 4 different characters, symbols or digits".into());
    }
    Ok(())
}

pub fn validate_generic_name<T: AsRef<str>>(name: T) -> Result<(), String> {
    let name = name.as_ref();
    if name.is_empty() {
        return Err("name is empty".into());
    }
    if name.len() > 255 {
        return Err("name is too long".into());
    }
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[^!<>=]+$").unwrap());
    if !RE.is_match(name) {
        return Err("name format is invalid".into());
    }
    Ok(())
}

pub fn is_email_other_taken(user_id: Option<i64>, email: &str, conn: &mut PgConnection) -> AppResult<bool> {
    let taken = if let Some(user_id) = user_id {
        diesel_exists!(
            emails::table
                .filter(lower(emails::value).eq(email.to_lowercase()))
                .filter(emails::user_id.ne(user_id)),
            conn
        )
    } else {
        diesel_exists!(
            emails::table.filter(lower(emails::value).eq(email.to_lowercase())),
            conn
        )
    };
    Ok(taken)
}

pub fn is_ident_name_other_taken(user_id: Option<i64>, ident_name: &str, conn: &mut PgConnection) -> AppResult<bool> {
    if let Some(user_id) = user_id {
        let query = users::table
        .filter(lower(users::ident_name).eq(ident_name.to_lowercase()))
        .filter(users::id.ne(user_id));
        Ok(diesel_exists!(query, conn))
    } else {
        let query = users::table.filter(lower(users::ident_name).eq(ident_name.to_lowercase()));
        Ok(diesel_exists!(query, conn))
    }
}
