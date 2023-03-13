pub mod pagination;
pub mod permit_filter;
pub mod url_filter;
mod delete;

pub use delete::*;
pub use pagination::*;
pub use permit_filter::*;

use diesel::expression::{is_aggregate, AppearsOnTable, ValidGrouping};
use diesel::pg::{Pg, PgConnection};
use diesel::prelude::*;
use diesel::query_builder::*;
use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};
use diesel::sql_types::*;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use once_cell::sync::OnceCell;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

pub static DB_POOL: OnceCell<PgPool> = OnceCell::new();
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

// pub fn connect()? -> PgConnection {
//     PgConnection::establish(&crate::database_url()).expect("connect database error")
// }
pub fn connect() -> Result<PooledConnection<ConnectionManager<PgConnection>>, PoolError> {
    // println!("==========get db conn");
    DB_POOL.get().unwrap().get()
}

pub fn build_pool(database_url: &str) -> Result<PgPool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    diesel::r2d2::Pool::builder()
        .max_size(crate::database_conns())
        .build(manager)
}

pub fn migrate(conn: &mut PgConnection) {
    println!(
        "Has pending migration: {}",
        conn.has_pending_migration(MIGRATIONS).unwrap()
    );
    conn.run_pending_migrations(MIGRATIONS)
        .expect("migrate db should worked");
}

pub struct AndQueryFragments(Vec<Box<dyn QueryFragment<Pg>>>);

impl Expression for AndQueryFragments {
    type SqlType = Bool;
}

impl<T> AppearsOnTable<T> for AndQueryFragments {}
impl ValidGrouping<()> for AndQueryFragments {
    type IsAggregate = is_aggregate::Never;
}

impl AndQueryFragments {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
impl QueryFragment<Pg> for AndQueryFragments {
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
        out.unsafe_to_cache_prepared();
        if !self.0.is_empty() {
            out.push_sql("(");
            for (i, fragment) in self.0.iter().enumerate() {
                fragment.walk_ast(out.reborrow())?;
                if i < self.0.len() - 1 {
                    out.push_sql(" AND ");
                }
            }
            out.push_sql(")");
        } else {
            out.push_sql("");
        }
        Ok(())
    }
}

pub struct OrQueryFragments(Vec<Box<dyn QueryFragment<Pg>>>);

impl Expression for OrQueryFragments {
    type SqlType = Bool;
}

impl<T> AppearsOnTable<T> for OrQueryFragments {}
impl ValidGrouping<()> for OrQueryFragments {
    type IsAggregate = is_aggregate::Never;
}

impl OrQueryFragments {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
impl QueryFragment<Pg> for OrQueryFragments {
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
        out.unsafe_to_cache_prepared();
        if !self.0.is_empty() {
            out.push_sql("(");
            for (i, fragment) in self.0.iter().enumerate() {
                fragment.walk_ast(out.reborrow())?;
                if i < self.0.len() - 1 {
                    out.push_sql(" OR ");
                }
            }
            out.push_sql(")");
        } else {
            out.push_sql("");
        }
        Ok(())
    }
}

sql_function!(fn lower(x: diesel::sql_types::Text) -> diesel::sql_types::Text);
