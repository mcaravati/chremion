use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use btleplug::api::{bleuuid::uuid_from_u16, BDAddr, Central, Peripheral, WriteType};
#[cfg(target_os = "linux")]
use btleplug::bluez::{adapter::Adapter, manager::Manager};
#[cfg(target_os = "macos")]
use btleplug::corebluetooth::{adapter::Adapter, manager::Manager};
#[cfg(target_os = "windows")]
use btleplug::winrtble::{adapter::Adapter, manager::Manager};
use serde::{Deserialize, Serialize};
use btleplug::Error;

#[derive(Deserialize, Serialize)]
struct Glasses {
    name: String,
}

#[derive(Debug, Clone, Copy)]
struct SharedData {
    glasses_address: Option<BDAddr>,
}

#[get("/display")]
async fn display(shared_data: web::Data<Arc<Mutex<SharedData>>>) -> impl Responder {
    let shared_data = shared_data.lock().unwrap();
    match shared_data.glasses_address {
        Some(..) => {
            println!("Not empty : {}", shared_data.glasses_address.unwrap());

            let manager = Manager::new().unwrap();
            let adapters = manager.adapters().unwrap();
            let central = adapters.into_iter().nth(0).unwrap();

            let glasses = central
                .peripheral(shared_data.glasses_address.unwrap());

            match glasses {
                None => {
                    println!("Glasses is empty")
                }
                Some(..) => {
                    if glasses.is_connected() {
                        match glasses.disconnect() {
                            Err(exception) => {
                                eprintln!("[-] {}", exception);
                            }
                            _ => {}
                        }
                    }
                }
            }

            HttpResponse::Ok()
        }
        None => {
            println!("Empty");
            HttpResponse::Conflict()
        }
    }
}

#[post("/connect")]
async fn connect(
    shared_data: web::Data<Arc<Mutex<SharedData>>>,
    data: web::Form<Glasses>,
) -> impl Responder {
    println!("Received request");
    let mut shared_data = shared_data.lock().unwrap();
    let data = data.0;

    // Get manager and get first adapter
    let manager = Manager::new().unwrap();
    let adapters = manager.adapters().unwrap();
    let central = adapters.into_iter().nth(0).unwrap();

    // Scan for 2 seconds
    central.start_scan().unwrap();
    thread::sleep(Duration::from_secs(2));

    // Try to find the glasses
    let glasses = central.peripherals().into_iter().find(|p| {
        p.properties()
            .local_name
            .iter()
            .any(|name| name.contains(&data.name))
    });

    match glasses {
        Some(..) => {
            println!("[+] Glasses found");
            let unwrapped_glasses = glasses.unwrap();

            match unwrapped_glasses.connect() {
                Ok(..) => {
                    unwrapped_glasses.discover_characteristics().unwrap();
                    shared_data.glasses_address = Some(unwrapped_glasses.address());

                    HttpResponse::Ok()
                }
                Err(exception) => {
                    eprintln!("[-] {}", exception);
                    HttpResponse::Conflict()
                }
            }
        }
        None => {
            println!("[-] The glasses couldn't be found");
            HttpResponse::Conflict()
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data = Arc::new(Mutex::new(SharedData {
        glasses_address: None,
    }));

    HttpServer::new(move || {
        App::new()
            .service(connect)
            .service(display)
            .data(data.clone())
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
