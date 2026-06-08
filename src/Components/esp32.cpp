#include "Arduino.h"
#include "Wire.h"
#include "WiFi.h"
#include "components/esp32.h"
#include <SPI.h>
#include "Components/lsm6dsr.h"
#include "Components/TCAL6408.h"
#include "Components/IIS2MDC.h"
#include "Components/BQ76905.h"
#include "Components/vl53l4cd.h"
#include "Components/tpl0102.h" 
#include "Components/tmp464.h"
#include "Components/fan.h"
#include "Components/iis2mdc.h"


Pins Esp32::pins;
HardwareConfig Esp32::hardwareConfig;
I2CConfig Esp32::i2cConfig;
SpiConfig Esp32::spiConfig;

void Esp32::initESP32() {
    Serial.begin(hardwareConfig.Serial_Clock);
    initPinStates();
    initSPI();
    initI2C();

    initSubComponents();
    initInterrupts();

    delay(1000);
    log_i("# ESP32 INIT DONE!");
}

void Esp32::initSubComponents() {
    TCAL6408::init();
    LSM6DSR::init();
    BQ76905::init();
    TMP464::init();
    IIS2MDC::init();
    TPL0102::init();
    Fan::init();
}

void Esp32::initPinStates() {
    
    pinMode(pins.PIN_FAN_EN, OUTPUT);
    pinMode(pins.PIN_IR_LED_0, OUTPUT);
    pinMode(pins.PIN_IR_LED_1, OUTPUT);
    pinMode(pins.PIN_IR_LED_2, OUTPUT);
    pinMode(pins.PIN_IR_LED_3, OUTPUT);
    pinMode(pins.PIN_LSM_INT_1, INPUT);
    pinMode(pins.PIN_LSM_INT_2, INPUT);
    pinMode(pins.PIN_VL53_0_INT, INPUT);
    pinMode(pins.PIN_VL53_1_INT, INPUT);
    pinMode(pins.PIN_VL53_2_INT, INPUT);
    pinMode(pins.PIN_BQ_ALERT, INPUT);

}


void Esp32::initSPI() {
    SPI.begin(pins.PIN_SCLK, pins.PIN_MISO, pins.PIN_MOSI, pins.PIN_LSM_CS);
}

void Esp32::initI2C() {
    Wire.begin(pins.PIN_SDA_0, pins.PIN_SCL_0, i2cConfig.I2C_Clock_0);  
    Wire.begin(pins.PIN_SDA_1, pins.PIN_SCL_1, i2cConfig.I2C_Clock_1);   
}


void Esp32::i2c_writeRegister(uint8_t deviceAddress, uint8_t registerAddress, uint8_t value) {
    Wire.beginTransmission(deviceAddress);
    Wire.write(registerAddress);
    Wire.write(value);

    uint8_t error = Wire.endTransmission();
    if (error != 0) {
        log_e("# I2C Error: %d", error);
    }
}

uint8_t Esp32::i2c_readRegister(uint8_t deviceAddress, uint8_t registerAddress) {
    Wire.beginTransmission(deviceAddress);
    Wire.write(registerAddress);
    if(Wire.endTransmission((uint8_t)false) != 0) {
        log_e("# I2C Error during register address transmission");
        return 0;

    }

    Wire.requestFrom((uint8_t)deviceAddress, (size_t)1);
    if (Wire.available()) {
        return Wire.read();
    } else {
        log_e("# I2C Error: No data available to read");
        return 0;
    }
}


uint16_t Esp32::i2c_readRegister16(uint8_t deviceAddress, uint8_t registerAddress) {
    Wire.beginTransmission(deviceAddress);
    Wire.write(registerAddress);
    if(Wire.endTransmission((uint8_t)false) != 0) {
        log_e("# I2C Error during register address transmission (16 bit)");
        return 0;
    }

    Wire.requestFrom((uint8_t)deviceAddress, (size_t)2);
    if (Wire.available() >= 2) {
        uint8_t highByte = Wire.read();
        uint8_t lowByte = Wire.read();
        return (highByte << 8) | lowByte;
    } else {
        log_e("# I2C Error: Not enough data available to read (16 bit)");
        return 0;
    }
}

void Esp32::i2c_writeRegister16(uint8_t deviceAddress, uint8_t registerAddress, uint16_t value) {
    Wire.beginTransmission(deviceAddress);
    Wire.write(registerAddress);
    Wire.write((value >> 8) & 0xFF); // High byte
    Wire.write(value & 0xFF);        // Low byte

    uint8_t error = Wire.endTransmission();
    if (error != 0) {
        log_e("# I2C Error: %d", error);
    }
}

void Esp32::spi_writeRegister(uint8_t registerAddress, uint8_t value, int PIN) {
    SPI.beginTransaction(spiConfig.spiSettings);
    digitalWrite(PIN, LOW);
    SPI.transfer(registerAddress & 0x7F); // Write
    SPI.transfer(value);
    digitalWrite(PIN, HIGH);
    SPI.endTransaction();
}

uint8_t Esp32::spi_readRegister(uint8_t registerAddress, int PIN) {
    byte data;
    SPI.beginTransaction(spiConfig.spiSettings);
    digitalWrite(PIN, LOW);
    SPI.transfer(registerAddress | 0x80); // Read
    data = SPI.transfer(0x00); // Dummy byte 
    digitalWrite(PIN, HIGH);
    SPI.endTransaction();
    return data;
}


void Esp32::initInterrupts() {
    attachInterrupt(digitalPinToInterrupt(pins.PIN_BQ_ALERT), BQ76905::alert, FALLING);
    attachInterrupt(digitalPinToInterrupt(pins.PIN_TCCAL_DRV_INT), TCAL6408::handleInterruptDriver, FALLING);
    
}

void Esp32::shutdown() {
    log_i("# SHUTTING DOWN ESP32");
    Serial.flush();

    WiFi.disconnect(true);
    WiFi.mode(WIFI_OFF);
    log_i("# ENTERING DEEP-SLEEP");
    delay(100);
    esp_deep_sleep_start();
}