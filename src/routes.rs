use crate::structures::{Device, ErrorMessage, Frame, SharedData};

use actix_web::{get, post, web, HttpResponse};

use crate::core::{
    chemion_connect, chemion_disconnect, chemion_discover, chemion_display, chemion_encode,
};
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

/// Displays a frame on the glasses
#[post("/display")]
pub async fn display(
    shared_data: web::Data<Arc<Mutex<SharedData>>>,
    data: web::Json<Frame>,
) -> HttpResponse {
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
    let data = data.0;

    // Call to the core function
    match chemion_display(&shared_data, data) {
        Ok(..) => HttpResponse::Ok().finish(),
        Err(error) => HttpResponse::InternalServerError().json(error),
    }
}

/// Encodes a model into a UART command
#[post("/encode")]
pub async fn encode(data: web::Json<Frame>) -> HttpResponse {
    match chemion_encode(data.0) {
        Ok(frame) => HttpResponse::Ok().json(frame),
        Err(error) => HttpResponse::InternalServerError().json(error),
    }
}

/// Discovers BLE peripherals with a name and returns it as a JSON array
#[get("/discover")]
pub async fn discover(shared_data: web::Data<Arc<Mutex<SharedData>>>) -> HttpResponse {
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

    match chemion_discover(&shared_data) {
        Ok(devices) => HttpResponse::Ok().json(devices),
        Err(error) => HttpResponse::InternalServerError().json(error),
    }
}

/// Disconnects from the current connected device
#[get("/disconnect")]
pub async fn disconnect(shared_data: web::Data<Arc<Mutex<SharedData>>>) -> HttpResponse {
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

    match chemion_disconnect(shared_data.deref_mut()) {
        Ok(..) => HttpResponse::Ok().finish(),
        Err(error) => HttpResponse::InternalServerError().json(error),
    }
}

/// Displays main UI, not needed if you don't want to use the API's GUI
#[get("/")]
pub async fn home(_shared_data: web::Data<Arc<Mutex<SharedData>>>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/home.html"))
}

#[get("/designer")]
pub async fn designer(_shared_data: web::Data<Arc<Mutex<SharedData>>>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/designer.html"))
}

/// Connects to a BLE device
#[post("/connect")]
pub async fn connect(
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

    match chemion_connect(shared_data.deref_mut(), data.0) {
        Ok(..) => HttpResponse::Ok().finish(),
        Err(error) => HttpResponse::InternalServerError().json(error),
    }
}
