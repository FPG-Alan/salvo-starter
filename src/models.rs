mod base;
// pub mod deploy;
// mod help;
// pub mod interflow;
// // pub mod paypal;
// pub mod product;
// pub mod questionnaire;
// pub mod sale;
// pub mod stock;
// pub mod stripe;
// mod studio;
// pub mod subscription;
// pub mod taxonomy;
// pub mod trade;
// pub mod wallet;

pub use base::*;
// pub use help::*;
// pub use studio::*;

pub static ID_NAME_SEARCH_TMPL: &str = "id::varchar(255)='{{data}}' or name ilike E'%{{data}}%'";
pub static ID_KIND_SEARCH_TMPL: &str = "id::varchar(255)='{{data}}' or kind ilike E'%{{data}}%'";
pub static ID_SEARCH_TMPL: &str = "id::varchar(255)='{{data}}'";
// for interflow_stream, which has `subject` field instead of `name` field
pub static ID_SUBJECT_SEARCH_TMPL: &str = "id::varchar(255)='{{data}}' or subject ilike E'%{{data}}%'";
pub static ID_VALUE_SEARCH_TMPL: &str = "id::varchar(255)='{{data}}' or value ilike E'%{{data}}%'";