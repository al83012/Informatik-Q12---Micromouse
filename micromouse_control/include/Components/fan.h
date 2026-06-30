#ifndef FAN_H
#define FAN_H
#include "esp32.h"

namespace Fan {

     void init();
     void setRotation(bool state);
     void enable();
     void disable();

}

#endif
