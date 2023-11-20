use warp::reject::Reject;

#[derive(Debug)]
pub enum Error {
    BadRequest,
    NotFound,
    InternalServerError,
    Unauthorized,
}
impl Reject for Error {}
