use log::info;
use std::convert::Infallible;
use warp::{http::Response, reject, Filter, Rejection, Reply};

mod handlers;
mod rejections;

pub fn bind_routes() -> impl Filter<Extract = (impl Reply,), Error = Infallible> + Clone {
    return handlers::handle_assets()
        .or(handlers::handle_login_routes())
        .or(handlers::handle_sign_up_routes())
        .or(handlers::handle_chat_routes())
        .recover(handle_rejection);
}

async fn handle_rejection(rej: Rejection) -> Result<impl Reply, Infallible> {
    if let Some(app_rej) = rej.find::<rejections::AppRejection>() {
        let mut resp = Response::builder().status(app_rej.status_code);
        let cloned_headers = app_rej.headers.clone();
        let resp_headers = resp.headers_mut().unwrap();

        for (name, value) in cloned_headers {
            if let Some(header_name) = name {
                resp_headers.insert(header_name, value);
            }
        }

        return Ok(resp.body(""));
    }

    if let Some(_) = rej.find::<reject::MethodNotAllowed>() {
        return Ok(Response::builder().status(405).body(""));
    }

    if rej.is_not_found() {
        return Ok(Response::builder().status(404).body(""));
    }

    // Yes, there are other built-in rejections, but we don't care about them here
    return Ok(Response::builder().status(500).body(""));
}
