#ifndef drv8424_h
#define drv8424_h

#include <Arduino.h>

namespace DRV8424 {
    extern volatile long encoderCount1;
    extern volatile long encoderCount2;
    extern volatile bool direction1;
    extern volatile bool direction2;

    extern uint8_t dutyCycle1;
    extern uint8_t dutyCycle2;
    extern int frequency;

    constexpr float maxCurrent = 0.3;
    constexpr float gearRatio = 2;
    constexpr float stepsPerRevolution_RAW = 32;
    constexpr float wheelDiameterCm = 2.3;
    constexpr float wheelCircumference = wheelDiameterCm * 3.14159;
    constexpr float stepsPerRevolution = stepsPerRevolution_RAW * gearRatio;
 
    
    
    void init(int frequency);
    bool assureSafeCurrent();
    void setDutyCycle1(int dutyCycle);
    void setDutyCycle2(int dutyCycle);
    void setSpeedPercentage1(float speedPercentage);
    void setSpeedPercentage2(float speedPercentage);

    void setFrequency(int frequency);

    void readEncoder1();
    void readEncoder2();

    void driveDistance(float distanceCm, float speedPercentage);
    long calculateTargetTicks(float distanceCm);
    void debugPrintEncoderCounts();


}
#endif
