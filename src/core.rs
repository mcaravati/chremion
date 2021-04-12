use crate::structures::{Device, ErrorMessage, Frame, SharedData};
use btleplug::api::{Central, Peripheral, WriteType};
use hex::decode;
use std::thread;
use std::time::Duration;

// Bluetooth dependencies for Linux
#[cfg(target_os = "linux")]
use btleplug::bluez::manager::Manager;

// Bluetooth dependencies for MacOS
#[cfg(target_os = "macos")]
use btleplug::corebluetooth::manager::Manager;

// Bluetooth dependencies for Windows
#[cfg(target_os = "windows")]
use btleplug::winrtble::manager::Manager;

/// Displays a frame on the currently connected glasses
///
/// # Arguments
///  * `shared_data` - The struct containing the address of the currently connected device, and the associated adapter
///  * `data` - The struct containing the 2D-array containing the UART command
///
pub fn chemion_display(shared_data: &SharedData, data: Frame) -> Result<(), ErrorMessage> {
    // Check if we've been connected to the glasses before
    match shared_data.glasses_address {
        Some(..) => {
            // Get bluetooth adapter
            let central = match shared_data.glasses_adapter.as_ref() {
                Some(adapter) => adapter,
                None => {
                    return Err(ErrorMessage {
                        message: String::from("Couldn't get bluetooth adapter"),
                    })
                }
            };

            // Get glasses
            let glasses = match central.peripheral(shared_data.glasses_address.unwrap()) {
                Some(glasses) => glasses,
                None => {
                    return Err(ErrorMessage {
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
                    for bytes_array in data.glasses_frame {
                        glasses
                            .write(&characteristic, &bytes_array, WriteType::WithoutResponse)
                            .unwrap();
                    }
                }
                None => {
                    return Err(ErrorMessage {
                        message: String::from("Couldn't find write characteristic"),
                    })
                }
            }

            Ok(())
        }
        None => Err(ErrorMessage {
            message: String::from("Please connect to a device first"),
        }),
    }
}

/// Encodes a frame into an UART command
///
/// # Arguments
///  * `data` - The struct containing the 2D-array containing the frame composed of (0|1|2|3)
///
pub fn chemion_encode(data: Frame) -> Result<Frame, ErrorMessage> {
    let mut binary_string = String::new();
    let mut uart = String::from("fa030039010006");
    let mut checksum: u8 = 7;
    let mut result: Vec<Vec<u8>> = Vec::new();

    for line in data.glasses_frame {
        for pixel in line {
            binary_string.push_str(match pixel {
                0 => "00",
                1 => "01",
                2 => "10",
                3 => "11",
                _ => {
                    return Err(ErrorMessage {
                        message: String::from("Wrong value in frame"),
                    })
                }
            });

            if binary_string.len() == 8 {
                let byte = isize::from_str_radix(&binary_string, 2).unwrap();
                checksum ^= byte as u8;
                uart.push_str(&format!("{:02x}", byte));
                binary_string = String::new();
            }
        }
    }

    uart.push_str(&format!("{:02x}", checksum));
    uart.push_str("55a9");

    let decoded = decode(uart).unwrap();
    let mut current_vec: Vec<u8> = Vec::new();

    for byte in decoded {
        current_vec.push(byte);

        if current_vec.len() == 20 {
            result.push(current_vec.clone());
            current_vec = Vec::new();
        }
    }

    result.push(current_vec.clone());

    Ok(Frame {
        glasses_frame: result,
    })
}

/// Discovers the BLE devices around
///
/// # Arguments
///  * `shared_data` - The struct containing the address of the currently connected device, and the associated adapter
///
pub fn chemion_discover(shared_data: &SharedData) -> Result<Vec<Device>, ErrorMessage> {
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
                    return Err(ErrorMessage {
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

    Ok(array)
}

/// Disconnects from the currently connected glasses, and resets the shared_data struct values
///
/// # Arguments
///  * `shared_data` - The struct containing the address of the currently connected device, and the associated adapter
///
pub fn chemion_disconnect(shared_data: &mut SharedData) -> Result<(), ErrorMessage> {
    // Disconnect glasses if address is stored
    return match shared_data.glasses_address {
        Some(glasses_address) => {
            let central = shared_data.glasses_adapter.as_ref().unwrap();

            // Try to find peripheral
            let peripheral = match central.peripheral(glasses_address) {
                Some(peripheral) => peripheral,
                None => {
                    return Err(ErrorMessage {
                        message: String::from("Couldn't get peripheral"),
                    })
                }
            };

            // Disconnect the glasses
            match peripheral.disconnect() {
                Ok(..) => {
                    shared_data.glasses_address = None;
                    shared_data.glasses_adapter = None;

                    Ok(())
                }
                Err(error) => {
                    eprintln!("[-] ERROR : {}", error);
                    Err(ErrorMessage {
                        message: String::from("Couldn't disconnect the glasses"),
                    })
                }
            }
        }
        None => Err(ErrorMessage {
            message: String::from("Couldn't get stored address"),
        }),
    };
}

/// Connects to the given glasses
///
/// # Arguments
///  * `shared_data` - The struct containing the address of the currently connected device, and the associated adapter
///  * `data` - The struct containing the information of the device to connect to
///
pub fn chemion_connect(shared_data: &mut SharedData, data: Device) -> Result<(), ErrorMessage> {
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
                    return Err(ErrorMessage {
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
                            return Err(ErrorMessage {
                                message: String::from("Couldn't connect to the glasses"),
                            });
                        }
                        _ => {}
                    };

                    // Store glasses' address and current adapter for future usages
                    shared_data.glasses_address = Some(glasses.address());
                    shared_data.glasses_adapter = Some(central);

                    Ok(())
                }
                Err(error) => {
                    eprintln!("[-] ERROR : {}", error);
                    Err(ErrorMessage {
                        message: String::from("Couldn't reach glasses"),
                    })
                }
            }
        }
        None => Err(ErrorMessage {
            message: String::from("Couldn't get peripheral"),
        }),
    }
}
