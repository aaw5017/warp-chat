#[macro_use]
extern crate lazy_static;

use async_once::AsyncOnce;
use dotenv::dotenv;
use sqlx::sqlite::SqlitePool;
use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tera::{Context, Tera};
use tokio::sync::{mpsc, RwLock};
use warp::{ws::Message, Filter, Rejection, Reply};

mod auth;
mod handlers;
mod middleware;
mod rejections;
mod websocket;

lazy_static! {
    pub static ref AEAD_KEY: [u8; 32] = {
        let aead_key_var = dotenv::var("AEAD_KEY").expect("AEAD_KEY not found in .env file!");
        match aead_key_var.as_bytes().try_into() {
            Ok(key) => {
                return key;
            }
            Err(e) => {
                eprintln!("Failure converting AEAD_KEY into array: {}", e);
                std::process::exit(1);
            }
        }
    };
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
    pub static ref TERA: Tera = {
        match Tera::new("src/templates/**/*.html") {
            Ok(tera) => {
                let mut t = tera;
                t.autoescape_on(vec![".html", ".sql"]);

                return t;
            }
            Err(e) => {
                eprintln!("Tera::new failed: {}", e);
                std::process::exit(1);
            }
        };
    };
}

type Clients = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

#[tokio::main]
async fn main() -> () {
    env_logger::init();
    dotenv().expect(".env file not found!");

    let clients = Clients::default();

    let with_clients = warp::any().map(move || {
        return clients.clone();
    });

    let assets = warp::get()
        .and(warp::path("assets"))
        .and(warp::fs::dir("src/assets/"));

    let index = warp::get().and(warp::path::end()).map(|| {
        let reply = warp::reply();
        let see_other = warp::reply::with_status(reply, warp::http::StatusCode::SEE_OTHER);
        return warp::reply::with_header(see_other, "Location", "/chat");
    });

    let chat = warp::get()
        .and(warp::path("chat"))
        .and(middleware::with_session_id_verification())
        .and_then(|| async {
            let ctx = Context::new();
            let render_result = TERA.render("chat.html", &ctx);

            if let Ok(html) = render_result {
                return Ok(warp::reply::html(html));
            }

            return Err(warp::reject());
        })
        .with(middleware::with_response_headers());

    let chat_ws = warp::ws()
        .and(warp::path("ws"))
        .and(with_clients)
        .map(websocket::handle);

    let login_page = warp::get()
        .and(warp::path("login"))
        .and_then(|| async {
            let ctx = Context::new();
            let render_result = TERA.render("login.html", &ctx);

            if let Ok(html) = render_result {
                return Ok(warp::reply::html(html));
            }

            return Err(warp::reject());
        })
        .with(middleware::with_response_headers());

    let sign_up_page = warp::get()
        .and(warp::path("sign-up"))
        .and_then(|| async {
            let ctx = Context::new();
            let render_result = TERA.render("sign_up.html", &ctx);

            if let Ok(html) = render_result {
                return Ok(warp::reply::html(html));
            }

            return Err(warp::reject());
        })
        .with(middleware::with_response_headers());

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::form())
        .and_then(handlers::handle_login)
        .and_then(middleware::with_new_session);

    let sign_up = warp::post()
        .and(warp::path("sign-up"))
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::form())
        .and_then(handlers::handle_signup)
        .and_then(middleware::with_new_session);

    let routes = index
        .or(chat)
        .or(login)
        .or(sign_up)
        .or(login_page)
        .or(sign_up_page)
        .or(assets)
        .or(chat_ws)
        .recover(handle_rejection);

    warp::serve(routes).run(([127, 0, 0, 1], 4040)).await;
}

async fn handle_rejection(rej: Rejection) -> Result<impl Reply, Infallible> {
    let status;
    let mut header = ("", "");

    // We need this custom matcher because warp's filter error tree priority is scuffed.
    // See GH issue and workaround: https://github.com/seanmonstar/warp/issues/77#issuecomment-959640913
    // TLDR; we can't use warp::reject() out of the box and expect it to take precedence over
    // METHOD_NOT_FOUND
    if let Some(err) = rej.find::<rejections::Error>() {
        match err {
            rejections::Error::BadRequest => {
                status = 400;
            }
            rejections::Error::NotFound => {
                status = 404;
            }
            rejections::Error::InternalServerError => {
                status = 500;
            }
            rejections::Error::Unauthorized => {
                status = 303;
                header = ("Location", "/login");
            }
        }
    } else if let Some(_) = rej.find::<warp::reject::MethodNotAllowed>() {
        status = 405;
    } else if rej.is_not_found() {
        status = 404;
    } else {
        status = 500;
    }

    let response = warp::http::Response::builder()
        .status(status)
        .header(header.0, header.1);
    return Ok(response.body(""));
}
