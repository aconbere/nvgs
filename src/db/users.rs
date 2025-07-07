use anyhow::{Result, anyhow};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use rusqlite::{Connection, params};

/* Represents a user that can authenticate to the api
 */
#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub password_hash: String,
}

impl User {
    pub fn new(username: &str, password: &str) -> Result<Self> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Error creating password hash: {}", e))?
            .to_string();
        Ok(User {
            username: username.to_string(),
            password_hash,
        })
    }

    pub fn verify_password(&self, password: &str) -> bool {
        let parsed_hash = match PasswordHash::new(&self.password_hash) {
            Ok(h) => h,
            _ => return false,
        };

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }
}

pub fn create_table(connection: &Connection) -> Result<()> {
    connection.execute(
        "
        CREATE TABLE users (
            username STRING NOT NULL,
            password_hash STRING NOT NULL,
            PRIMARY KEY (username)
        )
        ",
        params![],
    )?;
    Ok(())
}

pub fn get(connection: &Connection, username: String) -> Result<User> {
    let mut statement = connection.prepare(
        "
        SELECT
            username, password_hash
        FROM
            users
        WHERE
            username = ?1
        LIMIT
            1
        ",
    )?;

    let result: User = statement.query_row(params![username], |row| {
        Ok(User {
            username: row.get(0)?,
            password_hash: row.get(1)?,
        })
    })?;
    Ok(result)
}

pub fn insert(connection: &Connection, user: &User) -> Result<()> {
    connection.execute(
        "INSERT INTO
            users (username, password_hash)
        VALUES
            (?1, ?2)
        ON CONFLICT
            (username)
        DO UPDATE
        SET
            password_hash = ?2
        ",
        params![user.username, user.password_hash],
    )?;
    Ok(())
}
