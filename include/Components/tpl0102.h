#ifndef TPL0102_H
#define TPL0102_H
#include <Wire.h>
// Potentiometer
namespace TPL0102 {
namespace ComponentVars {
    //All address pins to GND
    constexpr uint8_t I2C_ADDRESS = 0x50;

    constexpr uint8_t REG_WIP_A = 0x00;
    constexpr uint8_t REG_WIP_B = 0x01;

    constexpr uint8_t REG_SETTINGS = 0x10;
    constexpr uint8_t REG_VOLMask = 0b10000000;
    constexpr uint8_t SETTING_SHDNMask = 0b01000000;
    constexpr uint8_t SETTING_WIPMask = 0b00100000;

   inline float highVoltage = 0.0f;
   inline uint8_t wiperPosA = 0;
   inline uint8_t wiperPosB = 0;
   inline uint8_t DefaultWiperPosA = 0;
   inline uint8_t DefaultWiperPosB = 0;
   inline bool shutdownEnabled = false;

    const uint16_t canWriteAutoRetryAttempts = 10;
    const uint16_t canWriteAutoRetryDelay = 50;
}

    void init(float highVoltage);
    int setVoltageA(float voltage);
    int setVoltageB(float voltage);
    int setDefaultVoltageA(float voltage);
    int setDefaultVoltageB(float voltage);

    float getVoltageA();
    float getVoltageB();
    float getDefaultVoltageA();
    float getDefaultVoltageB();
        
    int enterShutdown();
    int exitShutdown();

    float getHighVoltage();

    void DbgPrintVoltages();

    int SetVolatileWiperA(uint8_t position);
    int SetVolatileWiperB(uint8_t position);
    int SetNonVolatileWiperA(uint8_t position);
    int SetNonVolatileWiperB(uint8_t position);

    int getWiperA();
    int getWiperB();

    int enableNonVolatileWriting();
    int disableNonVolatileWriting();

    int canWrite();
    int canWriteAutoRetry();
        




}


#endif