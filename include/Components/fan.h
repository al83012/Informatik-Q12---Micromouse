#ifndef FAN_H
#define FAN_H
#include "esp32.h"

class Fan {
public:
    static void init();
    static void setRotation(bool state);
    static void enable();
    static void disable();

};

#endif
