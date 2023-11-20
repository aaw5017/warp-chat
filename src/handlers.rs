use crate::{auth, rejections};
use std::collections::HashMap;
use warp::Rejection;

mod db;

pub async fn handle_login(form: HashMap<String, String>) -> Result<String, Rejection> {
    let maybe_email = form.get("email");
    let maybe_pw = form.get("password");

    if maybe_email.is_none() || maybe_pw.is_none() {
        return Err(rejections::Error::BadRequest.into());
    }

    let email = maybe_email.unwrap();
    let pw = maybe_pw.unwrap();

    if let Ok(user) = db::get_user_by_email(email).await {
        if auth::verify_password(&user.hashed_password, &pw).is_ok() {
            // new token pair
            if let Ok(token_pair) = auth::get_new_token_pair() {
                if let Ok(created_session_id) = db::refresh_user_session(user.id, token_pair).await
                {
                    return Ok(created_session_id);
                }
            }

            return Err(rejections::Error::InternalServerError.into());
        }
    }

    return Err(rejections::Error::NotFound.into());
}

pub async fn handle_signup(form: HashMap<String, String>) -> Result<String, Rejection> {
    if let (Some(handle), Some(email), Some(password)) =
        (form.get("handle"), form.get("email"), form.get("password"))
    {
        let hashed_pw = match auth::get_new_hashed_password(password) {
            Ok(hashed) => hashed,
            Err(_) => {
                return Err(rejections::Error::InternalServerError.into());
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

        return Err(rejections::Error::InternalServerError.into());
    }

    return Err(rejections::Error::BadRequest.into());
}
