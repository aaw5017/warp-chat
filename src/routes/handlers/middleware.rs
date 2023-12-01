use crate::routes::rejections::AppRejection;
use warp::{
    http::{
        header::{HeaderMap, HeaderValue},
        Response,
    },
    reply::with::WithHeaders,
    Rejection, Reply,
};

pub fn with_response_headers() -> WithHeaders {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("text/html; charset=UTF-8"),
    );
    headers.insert(
        "Cross-Origin-Embedder-Policy",
        HeaderValue::from_static("require-corp"),
    );
    headers.insert(
        "Cross-Origin-Opener-Policy",
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        "Cross-Origin-Resource-Policy",
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        "X-CONTENT-TYPE-OPTIONS",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("X-FRAME-OPTIONS", HeaderValue::from_static("DENY"));
    headers.insert("X-XSS-PROTECTION", HeaderValue::from_static("0"));

    return warp::reply::with::headers(headers);
}

pub async fn with_new_session(session_id: String) -> Result<impl Reply, Rejection> {
    let maybe_cookie =
        HeaderValue::from_str(format!("id={}; HttpOnly; SameSite=Strict", session_id).as_str());

    if let Ok(cookie_value) = maybe_cookie {
        return Ok(Response::builder()
            .status(303)
            .header("Set-Cookie", cookie_value)
            .header("Location", "/chat")
            .body("")
            .unwrap());
    }

    return Err(AppRejection::new(None, 500).into());
}
