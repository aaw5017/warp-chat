#[macro_use]
extern crate lazy_static;

use dotenv::dotenv;
use tokio::signal;

mod routes;

#[tokio::main]
async fn main() -> () {
    tokio::spawn(async {
        env_logger::init();
        dotenv().expect(".env file not found!");

        warp::serve(routes::bind_routes())
            .run(([127, 0, 0, 1], 4040))
            .await;
    });

    match signal::ctrl_c().await {
        Ok(()) => println!("Shutting down..."),
        Err(e) => eprintln!("Unable to listen for ctrl_c signal!: {}", e),
    }
}
