#include "Components/VL53L4CD.h"
// Time-of-Flight (ToF) Distance Sensor
#include "VL53L4CD.h"
#include "Components/TCAL6408.h"

namespace TOF {
    
    void init() {
        
        log_d("# (TOF) Resetting sensors...");
        TCAL6408::shutdownVl53L_0();
        TCAL6408::shutdownVl53L_1();
        TCAL6408::shutdownVl53L_2();
        sensorFront.setTimeout(500);
        sensorLeft.setTimeout(500);
        sensorRight.setTimeout(500);
        TCAL6408::setToFToInput();
        
        for(uint8_t i = 0; i < sensorCount; i++) {
            log_d("# (TOF) Starting sensor %d...", i);
            if(sensors[i].init()) {
                log_i("# (TOF) Sensor %d initialized successfully.", i);
            } else {
                log_e("# (TOF) Error initializing sensor %d.", i);
            }

            sensors[i].setAddress(0x2A + i);
            sensors[i].startContinuous();

        }

        

    }

    void debugReadAllSensors() {
         for (uint8_t i = 0; i < sensorCount; i++)
    {
        Serial.print(sensors[i].read());
        if (sensors[i].timeoutOccurred()) { Serial.print(" TIMEOUT"); }
        Serial.print('\t');
    } 
    }


}