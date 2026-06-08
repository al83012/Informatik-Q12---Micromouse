#ifndef I2CTOOL_H
#define I2CTOOL_H
#include "Wire.h"
#include "Arduino.h"

struct I2CConfig
{
    int I2C_Clock_0 = 400000;
    int I2C_Clock_1 = 400000;

};


class I2CTOOL{ 
    public:
    static I2CConfig i2cConfig;
    static void init();
    static void i2c_writeRegister(uint8_t deviceAddress, uint8_t registerAddress, uint8_t value);
    static uint8_t i2c_readRegister(uint8_t deviceAddress, uint8_t registerAddress);

    static void i2c_writeRegister16(uint8_t deviceAddress, uint8_t registerAddress, uint16_t value);
    static uint16_t i2c_readRegister16(uint8_t deviceAddress, uint8_t registerAddress);
    static void I2CScanner();
};

#endif
