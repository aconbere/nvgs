use anyhow::Result;
use tokio_rusqlite::Connection;

use crate::db::users;

#[derive(Clone)]
pub struct Backend {
    connection: Connection,
}

impl Backend {
    pub fn new(connection: Connection) -> Self {
        Backend { connection }
    }

    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<users::User>> {
        let un = username.to_string();
        let user = self
            .connection
            .call(|conn| Ok(users::get(&conn, un).ok()))
            .await
            .ok()
            .flatten();

        match &user {
            None => return Ok(None),
            Some(user) => {
                if !user.verify_password(password) {
                    return Ok(None);
                }
            }
        }

        Ok(user)
    }

    #[allow(dead_code)]
    pub async fn get_user(&self, username: &str) -> Result<Option<users::User>> {
        let un = username.to_string();
        let user = self
            .connection
            .call(|conn| Ok(users::get(&conn, un).ok()))
            .await
            .ok()
            .flatten();

        Ok(user)
    }
}
