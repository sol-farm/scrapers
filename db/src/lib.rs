#![allow(clippy::needless_lifetimes)]
#![allow(clippy::bool_assert_comparison)]
#![allow(clippy::too_many_arguments)]
#![deny(unused_must_use)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derives_extra;
extern crate diesel_derives_traits;
#[macro_use]
extern crate diesel_migrations;
diesel_migrations::embed_migrations!("migrations");

pub mod client;
pub mod defaults;
pub mod filters;
pub mod models;
pub mod schema;
pub mod test_utils;
pub mod twap_pricing;

use anyhow::Result;
use diesel::prelude::*;

pub fn run_migrations(conn: &PgConnection) -> Result<()> {
    embedded_migrations::run_with_output(conn, &mut std::io::stdout())?;
    Ok(())
}
