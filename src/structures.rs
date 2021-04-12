use btleplug::api::BDAddr;

// Bluetooth dependencies for Linux
#[cfg(target_os = "linux")]
use btleplug::bluez::adapter::Adapter;

// Bluetooth dependencies for MacOS
#[cfg(target_os = "macos")]
use btleplug::corebluetooth::adapter::Adapter;

// Bluetooth dependencies for Windows
#[cfg(target_os = "windows")]
use btleplug::winrtble::adapter::Adapter;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Frame {
    pub glasses_frame: Vec<Vec<u8>>,
}

/// Glasses struct to receive the glasses' name from HTML forms
#[derive(Deserialize, Serialize)]
pub struct Device {
    pub device_name: String,
    pub device_address: String,
}

/// Shared data between Actix-static routes
#[derive(Clone)]
pub struct SharedData {
    pub glasses_address: Option<BDAddr>,
    pub glasses_adapter: Option<Adapter>,
}

/// JSON error message
#[derive(Serialize)]
pub struct ErrorMessage {
    pub message: String,
}
