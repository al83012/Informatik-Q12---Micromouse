#ifndef FAN_H
#define FAN_H
#include "esp32.h"

namespace FAN {

     constexpr float maxCurrent = 0.3;
     extern uint8_t dutyCycle;
     extern int frequency;

     void init();
     void setRotation(bool state);
     void enable();
     void disable();
     bool assureSafeCurrent();
     void setDutyCycle(int dutyCycle);
     void setSpeedPercentage(float speedPercentage);
     void setFrequency(int frequency);
    

}

#endif
