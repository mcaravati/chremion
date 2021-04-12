# Chremion
A basic API written in Rust for the CHEMION glasses. \
This program is entirely based on the research conducted in the [ChemionHacking](https://github.com/gsuberland/ChemionHacking) repository.\
This program is a WIP.

## Build
Build the program using 
```bash
$ cargo build
```

## Endpoints
### Endpoint : `/discover`
**Method :** GET \
Returns the BLE devices without a name. \
**Example :** \
**OUTPUT :**
```json
[{"device_name":"Glasses-0E:1B","device_address":"F7:E6:B5:7B:0E:1B"}]
```

### Endpoint : `/connect`
**Method :** POST \
Connects to the given BLE device. \
**Example :** \
**INPUT :**
```http request
device_name=Glasses-0E%3A1B&device_address=F7%3AE6%3AB5%3A7B%3A0E%3A1B
```
**THIS WILL BE CHANGED TO A JSON ENDPOINT**

### Endpoint : `/disconnect`
**Method :** GET \
Disconnects from the current device.

### Endpoint : `/encode`
**Method :** POST \
Encodes a model to an UART command \
**Example :** \
**INPUT :**
```json
{"glasses_frame":[[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]]}
```
**OUPUT :**
```json
{"glasses_frame":[[250,3,0,57,1,0,6,0,0,0,0,0,0,0,0,0,0,0,0,0],[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],[0,7,85,169]]}
```

### Endpoint : `/display`
**Method :** POST \
Displays the given frame to the glasses \
**Example :** \
**INPUT :**
```json
{"glasses_frame":[[250,3,0,57,1,0,6,0,0,0,0,0,0,0,0,0,0,0,0,0],[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],[0,7,85,169]]}
```

### Error handling
On error, the server returns an HTTP error 500. \
All the error messages are following this template :
```json
{"message":"Error message"}
```

## TODO
 - Create a web-based matrix designer
 - Write endpoints' documentation
 - Add configuration file
 - Make a good UI
 - Create tests
 - Hover display
 - Add error fields on all pages
 - Disconnect on Ctrl+C

## Contributors
 - mcaravati