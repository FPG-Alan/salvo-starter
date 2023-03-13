use chrono::{Duration, Utc};
use diesel::prelude::*;

use crate::email::send_email_with_tmpl;
use crate::models::*;
use crate::schema::*;
use crate::{db, things, AppResult};


// pub fn avatar_base_dir(id: i64, abs: bool) -> String {
//     if abs {
//         join_path!(&crate::space_path(), "users", &id.to_string(), "avatars")
//     } else {
//         join_path!("users", &id.to_string(), "avatars")
//     }
// }

impl User {
    // pub fn avatar_base_dir(&self, abs: bool) -> String {
    //     avatar_base_dir(self.id, abs)
    // }
   
    pub async fn send_verification_email(&self, address: &str) -> AppResult<()> {
        let code_value = crate::generate_digit_code(6);
        let code = NewSecurityCode {
            user_id: self.id,
            email: Some(address),
            value: &code_value,
            send_method: "email",
            expired_at: Utc::now() + Duration::hours(12),
            updated_by: Some(self.id),
            created_by: Some(self.id),
        };
        let mut conn = db::connect()?;
        let query = security_codes::table
            .filter(security_codes::user_id.eq(self.id))
            .filter(security_codes::created_at.ge(Utc::now() - Duration::minutes(1)));
        if diesel_exists!(query, &mut conn) {
            return Err(crate::Error::FrequentlyRequest);
        }
        diesel::delete(
            security_codes::table
                .filter(security_codes::user_id.eq(self.id))
                .filter(security_codes::send_method.eq(&code.send_method)),
        )
        .execute(&mut conn)?;
        diesel::insert_into(security_codes::table)
            .values(&code)
            .execute(&mut conn)?;

        let data = things::notification::user::VerificationContext {
            recipient: self,
            token: code.value,
        };
        send_email_with_tmpl(
            vec![address.to_owned()],
            "Please verify your email address",
            "verification",
            &data,
        )
        .await
    }
    pub async fn send_security_code_email(&self, address: &str) -> AppResult<()> {
        let code_value = crate::generate_digit_code(6);
        let code = NewSecurityCode {
            user_id: self.id,
            value: &code_value,
            email: Some(address),
            send_method: "email",
            expired_at: Utc::now() + Duration::minutes(60),
            updated_by: Some(self.id),
            created_by: Some(self.id),
        };
        let mut conn = db::connect()?;
        let query = security_codes::table
            .filter(security_codes::user_id.eq(self.id))
            .filter(security_codes::created_at.ge(Utc::now() - Duration::minutes(1)));
        if diesel_exists!(query, &mut conn) {
            return Err(crate::Error::FrequentlyRequest);
        }
        diesel::delete(security_codes::table.filter(security_codes::user_id.eq(self.id))).execute(&mut conn)?;
        diesel::insert_into(security_codes::table)
            .values(&code)
            .execute(&mut conn)?;
        
        let data = things::notification::user::SecurityCodeContext {
            code: code_value,
            recipient: self,
        };
        send_email_with_tmpl(vec![address.to_owned()], "Security code", "security_code", &data).await
    }
}
