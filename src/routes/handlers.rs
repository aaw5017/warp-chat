use crate::routes::rejections::AppRejection;
use std::collections::HashMap;
use tera::{Context, Tera};
use warp::{
    http::header::{HeaderMap, HeaderValue},
    Filter, Rejection, Reply,
};

mod auth;
mod db;
mod middleware;
mod websocket;

lazy_static! {
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

async fn handle_login(form: HashMap<String, String>) -> Result<String, Rejection> {
    if let (Some(email), Some(password)) = (form.get("email"), form.get("password")) {
        if let Ok(user) = db::get_user_by_email(email).await {
            if auth::verify_password(&user.hashed_password, &password).is_ok() {
                // new token pair
                if let Ok(token_pair) = auth::get_new_token_pair() {
                    if let Ok(created_session_id) =
                        db::refresh_user_session(user.id, token_pair).await
                    {
                        return Ok(created_session_id);
                    }
                }

                return Err(AppRejection::new(None, 500).into());
            }
        }
    }

    return Err(AppRejection::default().into());
}

async fn handle_signup(form: HashMap<String, String>) -> Result<String, Rejection> {
    if let (Some(handle), Some(email), Some(password)) =
        (form.get("handle"), form.get("email"), form.get("password"))
    {
        let hashed_pw = match auth::get_new_hashed_password(password) {
            Ok(hashed) => hashed,
            Err(_) => {
                return Err(AppRejection::new(None, 500).into());
            }
        };

        // get token pair
        if let Ok(token_pair) = auth::get_new_token_pair() {
            if let Ok(session_id) =
                db::create_new_user(&email, &handle, &hashed_pw, token_pair).await
            {
                return Ok(session_id);
            }
        }

        return Err(AppRejection::new(None, 500).into());
    }

    return Err(AppRejection::default().into());
}

async fn handle_chat(maybe_cookie: Option<String>) -> Result<impl Reply, Rejection> {
    if let Some(session_cookie) = maybe_cookie {
        if auth::verify_cookie(&session_cookie).is_ok() {
            if let Some(user_session) = db::get_session(&session_cookie).await {
                let mut ctx = Context::new();
                ctx.insert("csrf_token", &user_session.csrf_token);

                if let Ok(html) = TERA.render("chat.html", &ctx) {
                    return Ok(warp::reply::html(html));
                }

                return Err(AppRejection::new(None, 500).into());
            }
        }
    }

    // TODO: Add location redirect headers here
    let mut headers = HeaderMap::new();
    headers.insert("Location", HeaderValue::from_static("/login"));
    return Err(AppRejection::new(Some(headers), 303).into());
}

pub fn handle_assets() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    return warp::get()
        .and(warp::path("assets"))
        .and(warp::fs::dir("src/assets/"));
}

pub fn handle_login_routes() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let login_page = warp::get()
        .and(warp::path("login"))
        .and_then(|| async {
            let ctx = Context::new();
            let render_result = TERA.render("login.html", &ctx);

            if let Ok(html) = render_result {
                return Ok(warp::reply::html(html));
            }

            let rej = AppRejection::new(None, 404);
            return Err(warp::reject::custom(rej));
        })
        .with(middleware::with_response_headers());

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::form())
        .and_then(handle_login)
        .and_then(middleware::with_new_session);

    return login_page.or(login);
}

pub fn handle_sign_up_routes() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let sign_up_page = warp::get()
        .and(warp::path("sign-up"))
        .and_then(|| async {
            let ctx = Context::new();
            let render_result = TERA.render("sign_up.html", &ctx);

            if let Ok(html) = render_result {
                return Ok(warp::reply::html(html));
            }

            let rej = AppRejection::new(None, 404);
            return Err(warp::reject::custom(rej));
        })
        .with(middleware::with_response_headers());

    let sign_up = warp::post()
        .and(warp::path("sign-up"))
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::form())
        .and_then(handle_signup)
        .and_then(middleware::with_new_session);

    return sign_up_page.or(sign_up);
}

pub fn handle_chat_routes() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let index = warp::get().and(warp::path::end()).map(|| {
        let reply = warp::reply();
        let see_other = warp::reply::with_status(reply, warp::http::StatusCode::SEE_OTHER);
        return warp::reply::with_header(see_other, "Location", "/chat");
    });

    let chat = warp::get()
        .and(warp::path("chat"))
        .and(warp::cookie::optional::<String>("id"))
        .and_then(handle_chat)
        .with(middleware::with_response_headers());

    let chat_ws = warp::ws().and(warp::path("ws")).map(websocket::handle);

    return index.or(chat).or(chat_ws);
}
