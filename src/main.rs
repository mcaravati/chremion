mod core;
mod routes;
mod structures;

use crate::routes::{connect, designer, disconnect, discover, display, encode, home};
use crate::structures::SharedData;
use actix_files::Files;
use actix_web::{App, HttpServer};
use std::sync::{Arc, Mutex};

/// Program entry point, runs the Actix-static app
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("[*] Server running on port 8000");

    // Initialize empty data
    let data = Arc::new(Mutex::new(SharedData {
        glasses_address: None,
        glasses_adapter: None,
    }));

    // Run web server
    HttpServer::new(move || {
        App::new()
            .service(connect)
            .service(display)
            .service(disconnect)
            .service(discover)
            .service(home)
            .service(encode)
            .service(designer)
            .service(Files::new("/", "./static/"))
            .data(data.clone())
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
