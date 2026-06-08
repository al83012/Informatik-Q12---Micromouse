#include "Components/IIS2MDC.h"
#include "Arduino.h"
#include "Components/esp32.h"
#include "i2ctool.h"
// 3D- Magnetometer
IIS2MDC_Componentvars iis2mdc_componentVars;

void IIS2MDC::init() {
    Wire.beginTransmission(iis2mdc_componentVars.I2C_ADDRESS);
    if (Wire.endTransmission() != 0) {
        log_e("# IIS2MDC (Magnetometer) not found");
    } else {
        log_i("# IIS2MDC (Magnetometer) initialized successfully");
    }


    I2CTOOL::i2c_writeRegister(iis2mdc_componentVars.I2C_ADDRESS, iis2mdc_componentVars.REG_CFG_A, iis2mdc_componentVars.DEFAULT_CONFIG_A);
    I2CTOOL::i2c_writeRegister(iis2mdc_componentVars.I2C_ADDRESS, iis2mdc_componentVars.REG_CFG_C, iis2mdc_componentVars.DEFAULT_CONFIG_C);


}

void IIS2MDC::reboot() {
   uint8_t val = I2CTOOL::i2c_readRegister(iis2mdc_componentVars.I2C_ADDRESS,iis2mdc_componentVars.REG_CFG_A);
   I2CTOOL::i2c_writeRegister(iis2mdc_componentVars.I2C_ADDRESS, iis2mdc_componentVars.REG_CFG_A, val | 0x40);
   delay(5); 
}

void IIS2MDC::softReset() {
   uint8_t val = I2CTOOL::i2c_readRegister(iis2mdc_componentVars.I2C_ADDRESS,iis2mdc_componentVars.REG_CFG_A);
   I2CTOOL::i2c_writeRegister(iis2mdc_componentVars.I2C_ADDRESS, iis2mdc_componentVars.REG_CFG_A, val | 0x20);
   delay(5); 
}
