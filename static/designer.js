const GLASSES_DISPLAY = []

const PIXEL_STATE = {
    FULL: "#ffffff",
    MID: "#669999",
    QUARTER: "#334d4d",
    OFF: "#000000"
}

let CURRENT_PIXEL_STATE = PIXEL_STATE.FULL;

/**
 * Pixel class
 */
class Pixel {
    constructor(dom) {
        this.dom = dom;
        this.dom.className = "pixel";
        this.state = PIXEL_STATE.OFF;
        this.dom.style.backgroundColor = PIXEL_STATE.OFF;
        this.disabled = false;

        $(this.dom).on("click", (event) => {
            if(this.disabled) return;

            this.changeState(this.state === PIXEL_STATE.OFF ? CURRENT_PIXEL_STATE : PIXEL_STATE.OFF);
            display();
        });
    }

    /**
     * Changes the state of the pixel
     * Please use the PIXEL_STATE enum
     *
     * @param state The state to be changed to
     */
    changeState(state) {
        this.state = state;
        this.dom.style.backgroundColor = state;
    }
}

/**
 * Initialization function ran at the page loading
 */
$(() => {
    $("#display_button").on("click", () => {
       display();
    });

    // Bind disconnect button
    $("#disconnect_button").on("click", () => {
            disconnect();
    });

    // Select pixel intensity to 100%
    $("#full_pixel_button").on("click", () => {
        CURRENT_PIXEL_STATE = PIXEL_STATE.FULL;
    });

    // Select pixel intensity to 50%
    $("#mid_pixel_button").on("click", () => {
        CURRENT_PIXEL_STATE = PIXEL_STATE.MID;
    });

    // Select pixel intensity to 25%
    $("#quarter_pixel_button").on("click", () => {
        CURRENT_PIXEL_STATE = PIXEL_STATE.QUARTER;
    });

    // Clear the designer's display
    $("#clear_display_button").on("click", () => clear_display());

    // Initialize HTML wrapper
    let glassesWrapper = document.createElement("div");
    glassesWrapper.id = "glassesWrapper";
    let editor = document.getElementById("glasses_editor");
    editor.appendChild(glassesWrapper);

    // Pixels to be disabled around the nose
    const pixelsToBeDisabled = [
        [7, 11],
        [7, 12],
        [8, 10],
        [8, 11],
        [8, 12],
        [8, 13]
    ];

    // Build 2D-array
    for(let row = 0; row < 9; row++) {
        // Create logic rows
        GLASSES_DISPLAY[row] = new Array(24).fill(undefined);

        // Create DOM rows
        let glasses_row = document.createElement("div");
        glasses_row.className = "row";
        glassesWrapper.appendChild(glasses_row);
    }

    // Fill rows
    for(let row = 0; row < GLASSES_DISPLAY.length; row++) {
        for(let column = 0; column < GLASSES_DISPLAY[0].length; column++) {
            let pixel = new Pixel(document.createElement("div"));

            GLASSES_DISPLAY[row][column] = pixel;
            glassesWrapper.childNodes.item(row).appendChild(pixel.dom);
        }
    }

    disableNosePixels();
});

function display() {
    let convertedDisplay = [];
    for (let rowIndex in GLASSES_DISPLAY) {
        let convertedRow = [];
        for (let pixelIndex in GLASSES_DISPLAY[rowIndex]) {
            switch (GLASSES_DISPLAY[rowIndex][pixelIndex].state) {
                case PIXEL_STATE.OFF:
                    convertedRow.push(0);
                    break;
                case PIXEL_STATE.QUARTER:
                    convertedRow.push(1);
                    break;
                case PIXEL_STATE.MID:
                    convertedRow.push(2);
                    break;
                case PIXEL_STATE.FULL:
                    convertedRow.push(3);
                    break;
            }
        }
        convertedDisplay.push(convertedRow);
    }

    $.ajax({
        url: '/encode',
        type: 'post',
        dataType: "json",
        contentType: "application/json",
        data: JSON.stringify({"glasses_frame": convertedDisplay}),
        success: (received_data) => {
            $.ajax({
                url: '/display',
                type: 'post',
                dataType: "json",
                contentType: "application/json",
                data: JSON.stringify({"glasses_frame": received_data.glasses_frame}),
                success: (data) => {

                },
                error: (data) => {
                    let json = JSON.parse(data.responseText);
                    $(".error_field").text(json.message);
                }
            });
        },
        error: (data) => {
            let json = JSON.parse(data.responseText);
            $(".error_field").text(json.message);
        }
    });
}

/**
 * AJAX request to disconnect from the current device
 */
function disconnect() {
    $.ajax({
        url: '/disconnect',
        type: 'get',
        success: () => { },
        error: (data) => {
            let json = JSON.parse(data.responseText);
            $(".error_field").text(json.message);
        }
    });
}

/**
 * Clear the designer's display
 */
function clear_display() {
    for (let rowIndex in GLASSES_DISPLAY) {
        for (let pixelIndex in GLASSES_DISPLAY[rowIndex]) {
            GLASSES_DISPLAY[rowIndex][pixelIndex].changeState(PIXEL_STATE.OFF);
        }
    }

    disableNosePixels();
    display();
}

function disableNosePixels() {
    // Pixels to be disabled around the nose
    const pixelsToBeDisabled = [
        [7, 11],
        [7, 12],
        [8, 10],
        [8, 11],
        [8, 12],
        [8, 13]
    ];

    // Disable pixels around the nose
    pixelsToBeDisabled.forEach(pixel => {
        let objectPixel = GLASSES_DISPLAY[pixel[0]][pixel[1]];
        objectPixel.dom.style.backgroundColor = "transparent";
        objectPixel.disabled = true;
    });
}