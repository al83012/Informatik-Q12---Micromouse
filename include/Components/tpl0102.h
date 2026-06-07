#ifndef TPL0102_H
#define TPL0102_H
#include <Wire.h>
// Potentiometer

struct TPL0102_ComponentVars {
    //All address pins to GND
    const uint8_t I2C_ADDRESS = 0x50;

    const uint8_t REG_POT_A = 0x00;
    const uint8_t REG_POT_B = 0x01;
    //Not sure about whether we need to write onto volatile / non-volatile memory; Would be beneficial though


};

extern TPL0102_ComponentVars tpl0102_componentVars;


class TPL0102 {
    public:
        static void init();
        //0 -> 0% | 256 -> 100%
        // max 100kOhm resistance
        static void writeWiperA(uint8_t value);
        static void writeWiperB(uint8_t value);

};



#endif