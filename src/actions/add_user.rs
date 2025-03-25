use anyhow::Result;
use rusqlite::Connection;

use crate::db::users;

pub fn add_user(connection: &Connection, username: &str, password: &str) -> Result<()> {
    let user = users::User::new(username, password)?;
    users::insert(connection, &user)?;

    Ok(())
}
