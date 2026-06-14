#ifndef I2CTOOL_H
#define I2CTOOL_H
#include "Wire.h"
#include "Arduino.h"
#include "measurement.h"
namespace I2CTOOL {

namespace Config
{
    constexpr int I2C_Clock_0 = 200000;
    constexpr int I2C_Clock_1 = 50000;
    constexpr int WRITERETRYATTEMPTS = 5;
    constexpr int WRITERETRYDELAY = 1;
    constexpr int READRETRYATTEMPTS = 5;
    constexpr int READRETRYDELAY = 1;

}

    void init();

    bool I2C0Write(uint8_t Address, uint8_t Register, uint8_t Data, bool AutoRetry = true);
    bool I2C0Write(uint8_t Address, uint8_t Register, uint16_t Data, bool AutoRetry = true);
    bool I2C0Write(uint8_t Address, uint8_t Register, const uint8_t* DataStart, size_t Size, bool AutoRetry = true);

    bool I2C1Write(uint8_t Address, uint8_t Register, uint8_t Data, bool AutoRetry = true);
    bool I2C1Write(uint8_t Address, uint8_t Register, uint16_t Data, bool AutoRetry = true);
    bool I2C1Write(uint8_t Address, uint8_t Register, const uint8_t* DataStart, size_t Size, bool AutoRetry = true);

    bool I2C0Read(uint8_t Address, uint8_t Register, uint8_t& ReadOutput, bool AutoRetry = true);
    bool I2C0Read(uint8_t Address, uint8_t Register, uint16_t& ReadOutput, bool AutoRetry = true);
    bool I2C0Read(uint8_t Address, uint8_t Register, uint8_t* ReadOutput, size_t Size, bool AutoRetry = true);

    bool I2C1Read(uint8_t Address, uint8_t Register, uint8_t& ReadOutput, bool AutoRetry = true);
    bool I2C1Read(uint8_t Address, uint8_t Register, uint16_t& ReadOutput, bool AutoRetry = true);
    bool I2C1Read(uint8_t Address, uint8_t Register, uint8_t* ReadOutput, size_t Size, bool AutoRetry = true);

    void flip(uint16_t& Data);
    void flip(uint8_t* Data, size_t Size);
    void I2CScanner();

    void findComponent(uint8_t I2C_ADDRESS);
    void findComponent(uint8_t I2C_ADDRESS, Measurement::Sensors::SensorNames SensorName);
    void findComponent(Measurement::Sensors::SensorNames SensorName);



   /* void i2c_writeRegister(uint8_t deviceAddress, uint8_t registerAddress, uint8_t value);
    uint8_t i2c_readRegister(uint8_t deviceAddress, uint8_t registerAddress);

    void i2c_writeRegister16(uint8_t deviceAddress, uint8_t registerAddress, uint16_t value);
    uint16_t i2c_readRegister16(uint8_t deviceAddress, uint8_t registerAddress);*/





   


    


}
#endif
