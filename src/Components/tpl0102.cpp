#include "Components/TPL0102.h"
#include "Arduino.h"
#include "Components/esp32.h"

// Potentiometer
TPL0102_ComponentVars tpl0102_componentVars;

void TPL0102::init() {
     Wire.beginTransmission(tpl0102_componentVars.I2C_ADDRESS);
    if (Wire.endTransmission() != 0) {
        Serial.println("# TPL0102 (Potentiometer) not found!");
    } else {
        Serial.println("# TPL0102 (Potentiometer) initialized successfully");
    }
}

void writeWiperA(uint8_t value) {
    Esp32::i2c_writeRegister(tpl0102_componentVars.I2C_ADDRESS ,tpl0102_componentVars.REG_POT_A, value);
}

void writeWiperB(uint8_t value) {
    Esp32::i2c_writeRegister(tpl0102_componentVars.I2C_ADDRESS ,tpl0102_componentVars.REG_POT_B, value);
}
