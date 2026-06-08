#ifndef TPL0102_H
#define TPL0102_H
#include <Wire.h>
// Potentiometer

struct TPL0102_ComponentVars {
    //All address pins to GND
    const uint8_t I2C_ADDRESS = 0x50;

    const uint8_t REG_WIP_A = 0x00;
    const uint8_t REG_WIP_B = 0x01;

    const uint8_t REG_SETTINGS = 0x10;
    const uint8_t REG_VOLMask = 0b10000000;
    const uint8_t SETTING_SHDNMask = 0b01000000;
    const uint8_t SETTING_WIPMask = 0b00100000;

    const float highVoltage = 0.0f;
    uint8_t wiperPosA = 0;
    uint8_t wiperPosB = 0;
    uint8_t DefaultWiperPosA = 0;
    uint8_t DefaultWiperPosB = 0;
    bool shutdownEnabled = false;

    const uint16_t canWriteAutoRetryAttempts = 10;
    const uint16_t canWriteAutoRetryDelay = 50;
};

extern TPL0102_ComponentVars tpl0102_componentVars;


class TPL0102 {
    public:
        static void init(float highVoltage);
        static int setVoltageA(float voltage);
        static int setVoltageB(float voltage);
        static int setDefaultVoltageA(float voltage);
        static int setDefaultVoltageB(float voltage);

        static float getVoltageA();
        static float getVoltageB();
        static float getDefaultVoltageA();
        static float getDefaultVoltageB();
        
        static int enterShutdown();
        static int exitShutdown();

        static float getHighVoltage();

        static void DbgPrintVoltages();
    private:

        static int SetVolatileWiperA(uint8_t position);
        static int SetVolatileWiperB(uint8_t position);
        static int SetNonVolatileWiperA(uint8_t position);
        static int SetNonVolatileWiperB(uint8_t position);

        static int getWiperA();
        static int getWiperB();

        static int enableNonVolatileWriting();
        static int disableNonVolatileWriting();

        static int canWrite();
        static int canWriteAutoRetry();
        

};



#endif