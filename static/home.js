let DEVICES_LIST = [];
let SELECTED_DEVICE = undefined;
let SELECTED_DOM_ELEMENT = undefined;

/**
 * Main function, executed at document's loading
 */
$(() => {
    discover_devices();

    // Get buttons
    let connect_button = $("#connect_button");
    let disconnect_button = $("#disconnect_button");

    // Disable buttons
    connect_button.addClass(".disabled");
    disconnect_button.addClass(".disabled");

    // Bind refresh button
    $("#refresh_button").on("click", () => discover_devices());

    // Save selected device and enable connection button
    $(".device_list").on("click", "li", (event) => {
        // Adding class won't work ://
        $(SELECTED_DOM_ELEMENT).css("background-color", "transparent");
        SELECTED_DOM_ELEMENT = event.target;
        SELECTED_DEVICE = DEVICES_LIST[parseInt(SELECTED_DOM_ELEMENT.attributes["deviceNumber"].nodeValue)];
        $(SELECTED_DOM_ELEMENT).css("background-color", "dimgrey");

        $("#connect_button").removeClass(".disabled");
    });

    // Connect if button isn't disabled
    connect_button.on("click", () => {
        if (!$("#connect_button").hasClass(".disabled") && SELECTED_DEVICE !== undefined) {
            connect();
        }
    });

    // Bind disconnect button
    disconnect_button.on("click", () => {
        disconnect();
    });
});

/**
 * AJAX request to disconnect from the current device
 */
function disconnect() {
    $.ajax({
        url: '/disconnect',
        type: 'get',
        success: () => {
            $(".device_name").text("");
            discover_devices();
        },
        error: (data) => {
            let error_field = $(".error_field");
            let json = JSON.parse(data.responseText);
            error_field.text(json.message);

            setTimeout(() => error_field.text(""), 2000);
        }
    });
}

/**
 * Connects to the currently selected device
 */
function connect() {
    let deviceToConnect = SELECTED_DEVICE;
    $.ajax({
        url: '/connect',
        type: 'post',
        data: SELECTED_DEVICE,
        success: () => {
            $(".device_name").text(`Connected to ${deviceToConnect.device_name}`);
        },
        error: (data) => {
            let error_field = $(".error_field");
            let json = JSON.parse(data.responseText);
            error_field.text(json.message);

            setTimeout(() => error_field.text(""), 2000);
        }
    });
}

/**
 * Discovers BLE devices with a name
 */
function discover_devices() {
    // Reset selection
    SELECTED_DEVICE = undefined;
    SELECTED_DOM_ELEMENT = undefined;

    $(".device_list").empty();
    $.ajax({
        url: '/discover',
        type: 'get',
        dataType: 'json',
        success: (data) => {
            DEVICES_LIST = data;
            $(data).each((index, device) => {
                let element = document.createElement("li");
                element.setAttribute("deviceNumber", index.toString());
                element.innerText = `${device.device_name} (${device.device_address})`;

                $(".device_list").append(element);
            });
        },
        error: (data) => {
            let error_field = $(".error_field");
            let json = JSON.parse(data.responseText);
            error_field.text(json.message);

            setTimeout(() => error_field.text(""), 2000);
        }
    });
}