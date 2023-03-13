use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use handlebars::{handlebars_helper, Handlebars};


pub fn register_common_helpers(handlebars: &mut Handlebars<'_>) {
    handlebars_helper!(format_money: |v: BigDecimal| v.with_scale(2).to_string());
    handlebars_helper!(format_datetime: |v: str, f: str| v.parse::<DateTime<Utc>>().unwrap().format(f).to_string());
    handlebars.register_helper("format_datetime", Box::new(format_datetime));
    handlebars.register_helper("format_money", Box::new(format_money));
}
