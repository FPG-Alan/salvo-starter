use handlebars::Handlebars;
use once_cell::sync::Lazy;
use serde::Serialize;

use crate::AppResult;




static HANDLEBARS: Lazy<Handlebars<'static>> = Lazy::new(|| {
    let mut reg = Handlebars::new();
    reg.register_template_file("layout", "conf/emails/layout.hbs").unwrap();
    reg.register_template_file("security_code", "conf/emails/security_code.hbs")
        .unwrap();
    reg.register_template_file("verification", "conf/emails/verification.hbs")
        .unwrap();
    crate::helpers::handlebars::register_common_helpers(&mut reg);
    reg
});

pub async fn send_email_with_tmpl<T>(recipients: Vec<String>, subject: &str, tpl_name: &str, data: T) -> AppResult<()>
where
    T: Serialize,
{
    send_email(recipients, subject, HANDLEBARS.render(tpl_name, &data)?).await
}

pub async fn send_email(recipients: Vec<String>, subject: &str, body: String) -> AppResult<()> {
    // crate::aws::ses::send_email(recipients, subject, body).await
    Ok(())
}
