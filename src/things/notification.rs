use handlebars::Handlebars;
use once_cell::sync::Lazy;
use serde::Serialize;
use crate::{ AppResult};

static HANDLEBARS: Lazy<Handlebars<'static>> = Lazy::new(|| {
    let mut reg = Handlebars::new();
    crate::helpers::handlebars::register_common_helpers(&mut reg);
    reg
});
pub mod user {
    use crate::models::*;
    #[derive(Serialize, Debug)]
    pub struct SecurityCodeContext<'a> {
        pub recipient: &'a User,
        pub code: String,
    }

    #[derive(Serialize, Debug)]
    pub struct VerificationContext<'a> {
        pub recipient: &'a User,
        pub token: &'a str,
    }
}

pub fn render_body<T>(tpl_name: &str, data: T) -> AppResult<String>
where
    T: Serialize,
{
    match HANDLEBARS.render(tpl_name, &data) {
        Ok(data) => Ok(data),
        Err(e) => {
            tracing::error!(error = ?e, tpl_name = %tpl_name, "render notification template error");
            Err(crate::Error::Internal("render notification template error".into()))
        }
    }
}
