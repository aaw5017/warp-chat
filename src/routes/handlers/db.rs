use async_once::AsyncOnce;
use serde::Serialize;
use sqlx::{sqlite::SqlitePool, Error, FromRow};

lazy_static! {
    pub static ref DB: AsyncOnce<SqlitePool> = AsyncOnce::new(async {
        let database_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL not found in ENV!");

        match SqlitePool::connect(&database_url).await {
            Ok(db_pool) => {
                return db_pool;
            }
            Err(e) => {
                eprintln!("Db pool connection errors: {}", e);
                std::process::exit(1);
            }
        }
    });
}

#[derive(Debug, FromRow, Serialize)]
pub struct Session {
    pub id: String,
    pub csrf_token: String,
    pub user_id: u32,
    pub created_at: i64,
}
#[derive(Debug, FromRow, Serialize)]
pub struct User {
    pub id: u32,
    pub email: String,
    pub handle: String,
    pub hashed_password: String,
}

pub async fn create_new_user(
    email: &str,
    handle: &str,
    hashed_pw: &str,
    token_pair: (String, String), // (csrf, cookie)
) -> Result<String, Error> {
    let mut transaction = DB.get().await.begin().await?;

    let created_user_id: i64 = sqlx::query_scalar!(
        r#"
        INSERT INTO users (email, handle, hashed_password)
        VALUES (?, ?, ?)
        RETURNING id;
    "#,
        email,
        handle,
        hashed_pw
    )
    .fetch_one(&mut *transaction)
    .await?;

    let created_session_id: String = sqlx::query_scalar!(
        r#"
        INSERT INTO sessions (id, csrf_token, user_id, created_at)
        VALUES (?, ?, ?, unixepoch())
        RETURNING id;
    "#,
        token_pair.1,
        token_pair.0,
        created_user_id
    )
    .fetch_one(&mut *transaction)
    .await?;

    transaction.commit().await?;

    return Ok(created_session_id);
}

pub async fn refresh_user_session(
    user_id: u32,
    token_pair: (String, String),
) -> Result<String, Error> {
    let mut transaction = DB.get().await.begin().await?;

    let created_session_id = sqlx::query_scalar!(
        r#"
        DELETE FROM sessions where user_id = ?;
        INSERT INTO sessions (id, csrf_token, user_id, created_at)
        VALUES (?, ?, ?, unixepoch())
        RETURNING id;
    "#,
        user_id,
        token_pair.1,
        token_pair.0,
        user_id
    )
    .fetch_one(&mut *transaction)
    .await?;

    transaction.commit().await?;

    return Ok(created_session_id);
}

pub async fn get_session(id: &str) -> Option<Session> {
    let db = DB.get().await;
    let query = r#"
                SELECT id, csrf_token, user_id, created_at
                FROM sessions
                WHERE id = ?
                LIMIT 1;
            "#;

    let result = sqlx::query_as::<_, Session>(query)
        .bind(id)
        .fetch_one(db)
        .await;

    match result {
        Ok(session) => {
            return Some(session);
        }
        _ => {
            return None;
        }
    }
}

pub async fn get_user_by_session(id: &str) -> Option<User> {
    let db = DB.get().await;
    let query = r#"
        SELECT u.id, u.email, u.handle, '' as hashed_password
        FROM sessions as s
        INNER JOIN users as u 
        ON s.user_id = u.id
        WHERE s.id = ?
        LIMIT 1;
    "#;
    let result = sqlx::query_as::<_, User>(query)
        .bind(id)
        .fetch_one(db)
        .await;

    match result {
        Ok(user) => {
            return Some(user);
        }
        _ => {
            return None;
        }
    }
}

pub async fn get_user_by_email(email: &str) -> Result<User, Error> {
    let db = DB.get().await;

    let query = r#"
        SELECT id, email, handle, hashed_password
        FROM users
        WHERE email = ?
        LIMIT 1;
    "#;

    let user = sqlx::query_as::<_, User>(query)
        .bind(email)
        .fetch_one(db)
        .await?;

    return Ok(user);
}
