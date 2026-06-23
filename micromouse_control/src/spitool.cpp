#include "spitool.h"
#include "SPI.h"
#include "Arduino.h"
#include "Components/esp32.h"



void SPITOOL::init() {
    SPI.begin(SCLK, MISO, MOSI, LSM_CS);
}



void SPITOOL::spi_writeRegister(uint8_t registerAddress, uint8_t value, int PIN) {
    SPI.beginTransaction(SPITOOL::Config::spiSettings);
    digitalWrite(PIN, LOW);
    SPI.transfer(registerAddress & 0x7F); // Write
    SPI.transfer(value);
    digitalWrite(PIN, HIGH);
    SPI.endTransaction();
}

uint8_t SPITOOL::spi_readRegister(uint8_t registerAddress, int PIN) {
    byte data;
    SPI.beginTransaction(SPITOOL::Config::spiSettings);
    digitalWrite(PIN, LOW);
    SPI.transfer(registerAddress | 0x80); // Read
    data = SPI.transfer(0x00); // Dummy byte 
    digitalWrite(PIN, HIGH);
    SPI.endTransaction();
    return data;
}

