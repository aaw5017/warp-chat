#[macro_use]
extern crate lazy_static;

use dotenv::dotenv;

mod routes;

#[tokio::main]
async fn main() -> () {
    env_logger::init();
    dotenv().expect(".env file not found!");

    warp::serve(routes::bind_routes())
        .run(([127, 0, 0, 1], 4040))
        .await;
}
