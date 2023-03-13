use diesel::expression::{is_aggregate, AppearsOnTable, ValidGrouping};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_builder::*;
use diesel::sql_types::*;

#[derive(QueryId)]
pub enum PermitFilter {
    Allowed,
    Denied,
    Query(Vec<Box<dyn QueryFragment<Pg>>>),
}

impl Expression for PermitFilter {
    type SqlType = Bool;
}

impl<T> AppearsOnTable<T> for PermitFilter {}
impl ValidGrouping<()> for PermitFilter {
    type IsAggregate = is_aggregate::Never;
}

impl QueryFragment<Pg> for PermitFilter {
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
        out.unsafe_to_cache_prepared();
        match self {
            PermitFilter::Denied => out.push_sql("false"),
            PermitFilter::Allowed => out.push_sql("true"),
            PermitFilter::Query(fragments) => {
                if !fragments.is_empty() {
                    out.push_sql("(");
                    for (i, fragment) in fragments.iter().enumerate() {
                        fragment.walk_ast(out.reborrow())?;
                        if i < fragments.len() - 1 {
                            out.push_sql(" OR ");
                        }
                    }
                    out.push_sql(")");
                } else {
                    out.push_sql("false");
                    tracing::error!("permit_filter fragments empty is not allowed");
                }
            }
        }
        Ok(())
    }
}
