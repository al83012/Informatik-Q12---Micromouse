#include "spitool.h"
#include "SPI.h"
#include "Arduino.h"
#include "Components/esp32.h"
SpiConfig SPITOOL::spiConfig;



void SPITOOL::init() {
    SPI.begin(pins.PIN_SCLK, pins.PIN_MISO, pins.PIN_MOSI, pins.PIN_LSM_CS);
}



void SPITOOL::spi_writeRegister(uint8_t registerAddress, uint8_t value, int PIN) {
    SPI.beginTransaction(spiConfig.spiSettings);
    digitalWrite(PIN, LOW);
    SPI.transfer(registerAddress & 0x7F); // Write
    SPI.transfer(value);
    digitalWrite(PIN, HIGH);
    SPI.endTransaction();
}

uint8_t SPITOOL::spi_readRegister(uint8_t registerAddress, int PIN) {
    byte data;
    SPI.beginTransaction(spiConfig.spiSettings);
    digitalWrite(PIN, LOW);
    SPI.transfer(registerAddress | 0x80); // Read
    data = SPI.transfer(0x00); // Dummy byte 
    digitalWrite(PIN, HIGH);
    SPI.endTransaction();
    return data;
}

