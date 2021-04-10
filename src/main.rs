use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use actix_files::Files;
use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use btleplug::api::{BDAddr, Central, Peripheral, WriteType};

// Bluetooth dependencies for Linux
#[cfg(target_os = "linux")]
use btleplug::bluez::{adapter::Adapter, manager::Manager};

// Bluetooth dependencies for MacOS
#[cfg(target_os = "macos")]
use btleplug::corebluetooth::{adapter::Adapter, manager::Manager};

// Bluetooth dependencies for Windows
#[cfg(target_os = "windows")]
use btleplug::winrtble::{adapter::Adapter, manager::Manager};

use serde::{Deserialize, Serialize};

/// Glasses struct to receive the glasses' name from HTML forms
#[derive(Deserialize, Serialize)]
struct Device {
    device_name: String,
    device_address: String,
}

/// Shared data between Actix-static routes
#[derive(Clone)]
struct SharedData {
    glasses_address: Option<BDAddr>,
    glasses_adapter: Option<Adapter>,
}

/// JSON error message
#[derive(Serialize)]
struct ErrorMessage {
    message: String,
}

/// Display "OwO ?" on the glasses (will be improved !)
#[get("/display")]
async fn display(shared_data: web::Data<Arc<Mutex<SharedData>>>) -> HttpResponse {
    // Try to get shared data
    let shared_data = match shared_data.lock() {
        Err(error) => {
            eprintln!("[-] ERROR : {}", error);
            return HttpResponse::InternalServerError().json(ErrorMessage {
                message: String::from("Couldn't get shared data"),
            });
        }
        Ok(shared_data) => shared_data,
    };

    // Check if we've been connected to the glasses before
    match shared_data.glasses_address {
        Some(..) => {
            // Get bluetooth adapter
            let central = match shared_data.glasses_adapter.as_ref() {
                Some(adapter) => adapter,
                None => {
                    return HttpResponse::InternalServerError().json(ErrorMessage {
                        message: String::from("Couldn't get bluetooth adapter"),
                    })
                }
            };

            // Get glasses
            let glasses = match central.peripheral(shared_data.glasses_address.unwrap()) {
                Some(glasses) => glasses,
                None => {
                    return HttpResponse::InternalServerError().json(ErrorMessage {
                        message: String::from("Couldn't get glasses"),
                    })
                }
            };

            // Find write characteristic
            let write_characteristic =
                glasses
                    .characteristics()
                    .into_iter()
                    .find(|characteristic| {
                        characteristic.uuid.to_string() == "6e400002-b5a3-f393-e0a9-e50e24dcca9e"
                    });

            // Send byte array if write characteristic is found, else send HTTP error 500
            match write_characteristic {
                Some(characteristic) => {
                    let command: Vec<Vec<u8>> = vec![
                        vec![
                            0xFA, 0x03, 0x00, 0x39, 0x01, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00,
                            0x00, 0x3C, 0x00, 0x00, 0x3C, 0x00, 0x3C, 0xF3,
                        ],
                        vec![
                            0x00, 0x00, 0xF3, 0x00, 0xC3, 0xF3, 0x3C, 0x0C, 0xF3, 0x00, 0xC3, 0xF3,
                            0x3C, 0xCC, 0xF3, 0x00, 0x0C, 0xF3, 0x3C, 0xCC,
                        ],
                        vec![
                            0xF3, 0x00, 0x0C, 0xF3, 0x3C, 0xCC, 0xF3, 0x00, 0x00, 0x3C, 0x0F, 0x30,
                            0x3C, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x00,
                        ],
                        vec![0x00, 0xC8, 0x55, 0xA9],
                    ];

                    for bytes_array in command {
                        glasses
                            .write(&characteristic, &bytes_array, WriteType::WithoutResponse)
                            .unwrap();
                    }
                }
                None => {
                    return HttpResponse::InternalServerError().json(ErrorMessage {
                        message: String::from("Couldn't find write characteristic"),
                    })
                }
            }

            HttpResponse::Ok().finish()
        }
        None => HttpResponse::InternalServerError().json(ErrorMessage {
            message: String::from("Please connect to a device first"),
        }),
    }
}

/// Discovers BLE peripherals with a name and returns it as a JSON array
#[get("/discover")]
async fn discover(shared_data: web::Data<Arc<Mutex<SharedData>>>) -> HttpResponse {
    // Try to get shared data
    let shared_data = match shared_data.lock() {
        Err(error) => {
            eprintln!("[-] ERROR : {}", error);
            return HttpResponse::InternalServerError().json(ErrorMessage {
                message: String::from("Couldn't get shared data"),
            });
        }
        Ok(shared_data) => shared_data,
    };

    // Get manager and get first adapter
    let manager = Manager::new().unwrap();
    let adapters = manager.adapters().unwrap();

    // Get stored bluetooth adapter
    let central = match &shared_data.glasses_adapter {
        Some(adapter) => adapter.clone(),
        None => {
            let adapter = match adapters.into_iter().nth(0) {
                Some(adapter) => adapter,
                None => {
                    return HttpResponse::InternalServerError().json(ErrorMessage {
                        message: String::from("Couldn't get adapter"),
                    })
                }
            };
            adapter
        }
    };

    // Scan for 2 seconds
    central.start_scan().unwrap();
    thread::sleep(Duration::from_secs(2));

    // Build list of BLE peripherals that have a name
    let mut array: Vec<Device> = Vec::new();
    central.peripherals().into_iter().for_each(|peripheral| {
        if peripheral.properties().local_name.is_some() {
            array.push(Device {
                device_address: peripheral.address().to_string(),
                device_name: peripheral.properties().local_name.unwrap(),
            });
        }
    });

    HttpResponse::Ok().json(array)
}

/// Disconnects from the current connected device
#[get("/disconnect")]
async fn disconnect(shared_data: web::Data<Arc<Mutex<SharedData>>>) -> HttpResponse {
    // Try to get shared data
    let mut shared_data = match shared_data.lock() {
        Err(error) => {
            eprintln!("[-] ERROR : {}", error);
            return HttpResponse::InternalServerError().json(ErrorMessage {
                message: String::from("Couldn't get shared data"),
            });
        }
        Ok(shared_data) => shared_data,
    };

    // Disconnect glasses if address is stored
    return match shared_data.glasses_address {
        Some(glasses_address) => {
            let central = shared_data.glasses_adapter.as_ref().unwrap();

            // Try to find peripheral
            let peripheral = match central.peripheral(glasses_address) {
                Some(peripheral) => peripheral,
                None => {
                    return HttpResponse::InternalServerError().json(ErrorMessage {
                        message: String::from("Couldn't get peripheral"),
                    })
                }
            };

            // Disconnect the glasses
            match peripheral.disconnect() {
                Ok(..) => {
                    shared_data.glasses_address = None;
                    shared_data.glasses_adapter = None;

                    HttpResponse::Ok().finish()
                }
                Err(error) => {
                    eprintln!("[-] ERROR : {}", error);
                    HttpResponse::InternalServerError().json(ErrorMessage {
                        message: String::from("Couldn't disconnect the glasses"),
                    })
                }
            }
        }
        None => HttpResponse::InternalServerError().json(ErrorMessage {
            message: String::from("Couldn't get stored address"),
        }),
    };
}

/// Displays main UI, not needed if you don't want to use the API's GUI
#[get("/")]
async fn home(_shared_data: web::Data<Arc<Mutex<SharedData>>>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/home.html"))
}

/// Connects to a BLE device
#[post("/connect")]
async fn connect(
    shared_data: web::Data<Arc<Mutex<SharedData>>>,
    data: web::Form<Device>,
) -> HttpResponse {
    // Try to get shared data
    let mut shared_data = match shared_data.lock() {
        Err(error) => {
            eprintln!("[-] ERROR : {}", error);
            return HttpResponse::InternalServerError().json(ErrorMessage {
                message: String::from("Couldn't get shared data"),
            });
        }
        Ok(shared_data) => shared_data,
    };

    // Unpack the form's data
    let data = data.0;

    // Get manager and get first adapter
    let manager = Manager::new().unwrap();
    let adapters = manager.adapters().unwrap();

    // Get stored bluetooth adapter
    let central = match &shared_data.glasses_adapter {
        Some(adapter) => adapter.clone(),
        None => {
            let adapter = match adapters.into_iter().nth(0) {
                Some(adapter) => adapter,
                None => {
                    return HttpResponse::InternalServerError().json(ErrorMessage {
                        message: String::from("Couldn't get adapter"),
                    })
                }
            };
            adapter
        }
    };

    // Scan for 2 seconds
    central.start_scan().unwrap();
    thread::sleep(Duration::from_secs(2));

    // Try to find the glasses
    let glasses = central
        .peripherals()
        .into_iter()
        .find(|peripheral| peripheral.properties().address.to_string() == data.device_address);

    // Connect to the glasses
    match glasses {
        Some(glasses) => {
            match glasses.connect() {
                Ok(..) => {
                    // Get characteristics or throw HTTP error 500
                    match glasses.discover_characteristics() {
                        Err(error) => {
                            eprintln!("[-] ERROR : {}", error);
                            return HttpResponse::InternalServerError().json(ErrorMessage {
                                message: String::from("Couldn't connect to the glasses"),
                            });
                        }
                        _ => {}
                    };

                    // Store glasses' address and current adapter for future usages
                    shared_data.glasses_address = Some(glasses.address());
                    shared_data.glasses_adapter = Some(central);

                    HttpResponse::Ok().finish()
                }
                Err(error) => {
                    eprintln!("[-] ERROR : {}", error);
                    HttpResponse::InternalServerError().json(ErrorMessage {
                        message: String::from("Couldn't reach glasses"),
                    })
                }
            }
        }
        None => HttpResponse::InternalServerError().json(ErrorMessage {
            message: String::from("Couldn't get peripheral"),
        }),
    }
}

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
            .service(Files::new("/", "./static/"))
            .data(data.clone())
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
