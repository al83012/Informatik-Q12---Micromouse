#include "Components/TMP464.h"
#include "Arduino.h"
#include "Components/esp32.h"
#include "i2ctool.h"

// Temperature Sensor
TMP464_ComponentVars tmp464_componentVars;

void TMP464::init() {
    // Initialize the TMP464 temperature sensor
    Wire.beginTransmission(tmp464_componentVars.I2C_ADDRESS);
    if (Wire.endTransmission() != 0) {
        log_e("# TMP464 not found at address 0x48");
    } else {
        log_i("# TMP464 initialized successfully");
    }
}

float TMP464::readLocalTemperature() {
    uint16_t rawTemp = I2CTOOL::i2c_readRegister16(tmp464_componentVars.I2C_ADDRESS, tmp464_componentVars.REG_LOCAL_TEMP);
    return convertToCelsius(rawTemp);
}

float TMP464::readRemoteTemperature(uint8_t channel) {
    //Channel 1 = 0x01 ; Channel 2 = 0x02 ; Channel 3 = 0x03 ; Channel 4 = 0x04
    uint16_t rawTemp = I2CTOOL::i2c_readRegister16(tmp464_componentVars.I2C_ADDRESS, tmp464_componentVars.REG_REMOTE_TEMP + (channel - 1)); 
    return convertToCelsius(rawTemp);
}

float TMP464::convertToCelsius(uint16_t rawValue) {

    int16_t signedTemp = (int16_t)rawValue; 
    return (signedTemp >> 3) * 0.0625f; //Shift 3 bits, since data is only stored in the upper 13 bits (resolution of 0.0625 - fixed)
}