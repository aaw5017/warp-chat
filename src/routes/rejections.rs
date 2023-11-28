use warp::{http::HeaderMap, reject::Reject};

#[derive(Debug)]
pub struct AppRejection {
    pub headers: HeaderMap,
    pub status_code: u16,
}
impl AppRejection {
    pub fn new(header_opt: Option<HeaderMap>, status_code: u16) -> Self {
        let headers = match header_opt {
            Some(val) => val,
            None => HeaderMap::new(),
        };

        return Self {
            headers,
            status_code,
        };
    }
}
impl Reject for AppRejection {}
impl Default for AppRejection {
    fn default() -> Self {
        return Self {
            headers: HeaderMap::new(),
            status_code: 400,
        };
    }
}
