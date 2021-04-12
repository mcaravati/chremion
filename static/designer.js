const GLASSES_DISPLAY = []

const PIXEL_STATE = {
    FULL: "#ffffff",
    MID: "#669999",
    QUARTER: "#334d4d",
    OFF: "#000000"
}

/**
 * Pixel class
 */
class Pixel {
    constructor(dom) {
        this.dom = dom;
        this.dom.className = "pixel";
        this.state = PIXEL_STATE.OFF;
        this.disabled = false;

        this.changeState(PIXEL_STATE.OFF);

        this.dom.addEventListener("click", () => {
            if(this.disabled) return;

            this.changeState(this.state === PIXEL_STATE.OFF ? PIXEL_STATE.FULL : PIXEL_STATE.OFF);
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

    // Initialize HTML wrapper
    let glassesWrapper = document.createElement("div");
    glassesWrapper.id = "glassesWrapper";
    $("#glasses_display").append(glassesWrapper);

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

    // Disable pixels around the nose
    pixelsToBeDisabled.forEach(pixel => {
        let objectPixel = GLASSES_DISPLAY[pixel[0]][pixel[1]];
        objectPixel.dom.style.backgroundColor = "transparent";
        objectPixel.disabled = true;
    });
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