#include "i2ctool.h"
#include "Components/Esp32.h"

I2CConfig I2CTOOL::i2cConfig;



void I2CTOOL::init() {
    Wire.begin(pins.PIN_SDA_0, pins.PIN_SCL_0, i2cConfig.I2C_Clock_0);  
    Wire.begin(pins.PIN_SDA_1, pins.PIN_SCL_1, i2cConfig.I2C_Clock_1);   
}


void I2CTOOL::i2c_writeRegister(uint8_t deviceAddress, uint8_t registerAddress, uint8_t value) {
    Wire.beginTransmission(deviceAddress);
    Wire.write(registerAddress);
    Wire.write(value);

    uint8_t error = Wire.endTransmission();
    if (error != 0) {
        log_e("# I2C Error: %d", error);
    }
}

uint8_t I2CTOOL::i2c_readRegister(uint8_t deviceAddress, uint8_t registerAddress) {
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


uint16_t I2CTOOL::i2c_readRegister16(uint8_t deviceAddress, uint8_t registerAddress) {
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

void I2CTOOL::i2c_writeRegister16(uint8_t deviceAddress, uint8_t registerAddress, uint16_t value) {
    Wire.beginTransmission(deviceAddress);
    Wire.write(registerAddress);
    Wire.write((value >> 8) & 0xFF); // High byte
    Wire.write(value & 0xFF);        // Low byte

    uint8_t error = Wire.endTransmission();
    if (error != 0) {
        log_e("# I2C Error: %d", error);
    }
}

void I2CTOOL::I2CScanner(){
    uint8_t Devices0 = 0;
    uint8_t Devices1 = 0;
    uint8_t error;
    uint8_t address;

    log_i("Scanning for I2C devices...");
    log_i("Scanning I2C0...");
    for(address = 1; address < 127; address++){
        Wire.beginTransmission(address);
        error = Wire.endTransmission();

        if(error == 0){
            Devices0++;
            if(address < 16){
                log_i("I2C0 device found at address 0x0%X", address);
            }
            else{
                log_i("I2C0 device found at address 0x%X", address);
            }
        }
        else if(error == 4){
            if(address < 16){
                log_e("I2C0 error at address 0x0%X", address);
            }
            else{
                log_e("I2C0 error at address 0x%X", address);
            }
        }
    }
    log_i("I2C0 devices found: %d", Devices0);

    log_i("Scanning I2C1...");
    for(address = 1; address < 127; address++){
        Wire1.beginTransmission(address);
        error = Wire1.endTransmission();

        if(error == 0){
            Devices1++;
            if(address < 16){
                log_i("I2C1 device found at address 0x0%X", address);
            }
            else{
                log_i("I2C1 device found at address 0x%X", address);
            }
        }
        else if(error == 4){
            if(address < 16){
                log_e("I2C1 error at address 0x0%X", address);
            }
            else{
                log_e("I2C1 error at address 0x%X", address);
            }
        }
    }
    log_i("I2C1 devices found: %d", Devices1);
}