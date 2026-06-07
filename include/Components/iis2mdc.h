#ifndef IIS2MDC_H
#define IIS2MDC_H
#include <Wire.h>
// 3D- Magnetometer
// Scale +/- 50 Gauss (Resolution: 0.0015 g / LSB)
struct IIS2MDC_Componentvars {
    const uint8_t I2C_ADDRESS = 0x1E;


    //TODO: Add offset registers in case we need to compensate enviromental effects
    
    // CFG_A : [COMP_TEMP] [REBOOT] [SOFT_RST] [LP/HP] [ODR1] [ODR0] [MD1] [MD0]
    // Default set to CompTemp = 1 | SOFT_RST Off | HP | 100Hz | Continuous ; TODO: We may need to change those later on
    // CFG_B does not seem to need any changes for now
    // Only the block-update bit should be changed in CFG_C
    const uint8_t REG_CFG_A = 0x60;
    const uint8_t REG_CFG_B = 0x61;
    const uint8_t REG_CFG_C = 0x62;
    const uint8_t REG_STATUS = 0x67;

    const uint8_t REG_INT_SOURCE = 0x64;
    const uint8_t REG_INT_THIS_L = 0x65;
    const uint8_t REG_INT_THIS_H = 0x66;

    const uint8_t REG_OUTX_L = 0x68;
    const uint8_t REG_OUTX_H = 0x69;
    const uint8_t REG_OUTY_L = 0x6A;
    const uint8_t REG_OUTY_H = 0x6B;
    const uint8_t REG_OUTZ_L = 0x6C;
    const uint8_t REG_OUTZ_H = 0x6D;

    const uint8_t REG_TEMP_OUT_L = 0x6E;
    const uint8_t REG_TEMP_OUT_H = 0x6F;



    const uint8_t RESET_CONFIG_A =   0b00100000;
    const uint8_t DEFAULT_CONFIG_A = 0b10001100; 
    const uint8_t DEFAULT_CONFIG_C = 0b00010000;
};
extern IIS2MDC_Componentvars iis2mdc_componentVars;

class IIS2MDC {
    public:
    static void init();
    static void softReset();
    static void reboot();
    static float readTemperature();


};
#endif