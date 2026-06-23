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

    I2C1Write(ComponentVars::I2C_ADDRESS,ComponentVars::REG_CFG_A, static_cast<uint8_t>(ComponentVars::DEFAULT_CONFIG_A));
    I2C1Write(ComponentVars::I2C_ADDRESS,ComponentVars::REG_CFG_C, static_cast<uint8_t>(ComponentVars::DEFAULT_CONFIG_C));


}

void IIS2MDC::reboot() {
   uint8_t val ;
   I2C1Read(ComponentVars::I2C_ADDRESS,ComponentVars::REG_CFG_A, val);
   I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_CFG_A, static_cast<uint8_t>(val | 0x40));
   delay(5); 
}

void IIS2MDC::softReset() {
   uint8_t val;
   I2C1Read(ComponentVars::I2C_ADDRESS,ComponentVars::REG_CFG_A, val);

   I2C1Write(ComponentVars::I2C_ADDRESS,ComponentVars::REG_CFG_A, static_cast<uint8_t>(val | 0x20));
   delay(5); 
}
