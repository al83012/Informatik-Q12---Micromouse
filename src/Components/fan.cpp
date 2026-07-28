#include "Components/TCAL6408.h"
#include "Components/fan.h"
#include "Components/tpl0102.h"

void FAN::init() {
    TCAL6408::setPinStateDriver(TCAL6408::ComponentVars::PIN_FAN_PH, HIGH); // Default -> Clockwise
    enable();
}

void FAN::setRotation(bool state) {
    TCAL6408::setFanRotation(state);
}

void FAN::enable() {
    TCAL6408::setFanAwake(true);
}

void FAN::disable() {
    TCAL6408::setFanAwake(false);
}

bool FAN::assureSafeCurrent() {
log_d("# (DRV8424) Assuring safe current...");
    return TPL0102::setCurrentLimitFan(maxCurrent);
}

void setDutyCycle() {

}

void FAN::setDutyCycle(int dutyCycle) {
    log_d("# (FAN) Setting duty cycle to %d", dutyCycle);
    dutyCycle = constrain(dutyCycle, 0, 255);
    
    analogWrite(FAN_EN, abs(dutyCycle));


}

void FAN::setFrequency(int frequency) {
    if(frequency < 0) {frequency = -frequency;}
    log_d("# (FAN) Setting frequency to %d", frequency);
    FAN::frequency = frequency;
    analogWriteFrequency(frequency);
}

void FAN::setSpeedPercentage(float speedPercentage) {
    log_d("# (FAN) Setting speed percentage 1 to %f", speedPercentage);
    speedPercentage = constrain(speedPercentage, -100.0f, 100.0f);
    if(speedPercentage < 0.0) {setRotation(false); speedPercentage = abs(speedPercentage);}
    int dutyCycle = static_cast<int>((speedPercentage / 100.0f) * 255);
    setDutyCycle(dutyCycle);
}