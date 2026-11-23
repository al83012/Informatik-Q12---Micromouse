#include "Components/IIS2MDC.h"
#include "Arduino.h"
#include "Components/esp32.h"
#include "i2ctool.h"
// 3D- Magnetometer
using namespace Measurement::Sensors;
using namespace IIS2MDC;
using namespace I2CTOOL;

void IIS2MDC::init() {
   
    findComponent(SensorNames::IIS2MDC_MAGNETOMETER);

    I2C0Write(ComponentVars::I2C_ADDRESS,ComponentVars::REG_CFG_A, static_cast<uint8_t>(ComponentVars::DEFAULT_CONFIG_A));
    I2C0Write(ComponentVars::I2C_ADDRESS,ComponentVars::REG_CFG_C, static_cast<uint8_t>(ComponentVars::DEFAULT_CONFIG_C));


}

void IIS2MDC::reboot() {
   uint8_t val ;
   I2C0Read(ComponentVars::I2C_ADDRESS,ComponentVars::REG_CFG_A, val);
   I2C0Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_CFG_A, static_cast<uint8_t>(val | 0x40));
   delay(5); 
}

void IIS2MDC::softReset() {
   uint8_t val;
   I2C0Read(ComponentVars::I2C_ADDRESS,ComponentVars::REG_CFG_A, val);

   I2C0Write(ComponentVars::I2C_ADDRESS,ComponentVars::REG_CFG_A, static_cast<uint8_t>(val | 0x20));
   delay(5); 
}

float IIS2MDC::getHeading() {
   uint8_t buffer[6]; // Holds X_L, X_H, Y_L, Y_H, Z_L, Z_H
    
    // Set the MSB (0x80) on the starting register to enable auto-increment 
    // for ST sensors during multi-byte reads.
    uint8_t startRegister = ComponentVars::REG_OUTX_L | 0x80;

    // Perform a single burst read of all 6 bytes at once
    if (!I2C0Read(ComponentVars::I2C_ADDRESS, startRegister, buffer, 6)) {
        log_e("# Failed to read magnetometer heading data!");
        return 0.0f; 
    }

    // Reconstruct 16-bit signed values for X and Y axes from the buffer
    int16_t x = (static_cast<int16_t>(buffer[1]) << 8) | buffer[0];
    int16_t y = (static_cast<int16_t>(buffer[3]) << 8) | buffer[2];

    // Compute heading using atan2
    float heading = atan2(static_cast<float>(y), static_cast<float>(x));

    // Normalize range from [-PI, PI] to [0, 2*PI]
    if (heading < 0.0f) {
        heading += 2.0f * 3.14159265358979323846f;
    }

    return heading;
    }