#include "Components/IIS2MDC.h"
#include "Arduino.h"
#include "Components/esp32.h"
#include "i2ctool.h"
// 3D- Magnetometer

void IIS2MDC::init() {
    Wire.beginTransmission(IIS2MDC::ComponentVars::I2C_ADDRESS);
    if (Wire.endTransmission() != 0) {
        log_e("# IIS2MDC (Magnetometer) not found");
    } else {
        log_i("# IIS2MDC (Magnetometer) initialized successfully");
    }


    I2CTOOL::I2C1Write(IIS2MDC::ComponentVars::I2C_ADDRESS, IIS2MDC::ComponentVars::REG_CFG_A, static_cast<uint8_t>(IIS2MDC::ComponentVars::DEFAULT_CONFIG_A));
    I2CTOOL::I2C1Write(IIS2MDC::ComponentVars::I2C_ADDRESS, IIS2MDC::ComponentVars::REG_CFG_C, static_cast<uint8_t>(IIS2MDC::ComponentVars::DEFAULT_CONFIG_C));


}

void IIS2MDC::reboot() {
   uint8_t val ;
   I2CTOOL::I2C1Read(IIS2MDC::ComponentVars::I2C_ADDRESS,IIS2MDC::ComponentVars::REG_CFG_A, val);
   I2CTOOL::I2C1Write(IIS2MDC::ComponentVars::I2C_ADDRESS, IIS2MDC::ComponentVars::REG_CFG_A, static_cast<uint8_t>(val | 0x40));
   delay(5); 
}

void IIS2MDC::softReset() {
   uint8_t val;
   I2CTOOL::I2C1Read(IIS2MDC::ComponentVars::I2C_ADDRESS,IIS2MDC::ComponentVars::REG_CFG_A, val);

   I2CTOOL::I2C1Write(IIS2MDC::ComponentVars::I2C_ADDRESS, IIS2MDC::ComponentVars::REG_CFG_A, static_cast<uint8_t>(val | 0x20));
   delay(5); 
}
