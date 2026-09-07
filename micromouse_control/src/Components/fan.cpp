#include "Components/TCAL6408.h"
#include "Components/fan.h"


void Fan::init() {
    TCAL6408::setPinStateDriver(TCAL6408::ComponentVars::PIN_FAN_PH, HIGH); // Default -> Clockwise
}

void Fan::setRotation(bool state) {
    TCAL6408::setFanRotation(state);
}

void Fan::enable() {
    TCAL6408::setFanAwake(true);
}

void Fan::disable() {
    TCAL6408::setFanAwake(false);
}
