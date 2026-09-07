#include "Components/VL53L4CD.h"
// Time-of-Flight (ToF) Distance Sensor
#include "VL53L4CD.h"
#include "Components/TCAL6408.h"
#include "colors.h"
namespace VL53L4CD_PHYSICAL {
    
    void init() {
        
        log_d("# (TOF) Resetting sensors...");
        TCAL6408::shutdownVl53L_0();
        TCAL6408::shutdownVl53L_1();
        TCAL6408::shutdownVl53L_2();
        sensorFront.setTimeout(500);
        //sensorLeft.setTimeout(500);
        //sensorRight.setTimeout(500);
        TCAL6408::setToFToInput();
        delay(500);
        sensorFront.setBus(&Wire);
        //sensorLeft.setBus(&Wire);
        //sensorRight.setBus(&Wire);

        /*
        for(uint8_t i = 0; i < sensorCount; i++) {
            log_d("# (TOF) Starting sensor %d...", i);
            if(sensors[i].init()) {
                log_i("# (TOF) Sensor %d initialized successfully.", i);
            } else {
                log_e("# (TOF) Error initializing sensor %d.", i);
            }

            sensors[i].setAddress(0x2A + i);
            sensors[i].startContinuous();
            delay(100);
        }*/

         log_d("# (TOF) Starting sensor %d...", 0);
            if(sensors[0].init()) {
                log_i("# (TOF) Sensor %d initialized successfully.", 0);
                sensors[0].startContinuous();
            } else {
                log_e("# (TOF) Error initializing sensor %d.", 0);
            }

           // sensors[0].setAddress(0x2A);
            

        

    }

    void debugReadAllSensors() {
         for (uint8_t i = 0; i < sensorCount; i++)
    {
        Serial.print(sensors[i].read());
        if (sensors[i].timeoutOccurred()) { Serial.print(" TIMEOUT"); }
        Serial.print('\t');
    } 
    }

    uint16_t debugReadSensor(int sensorIndex) {
        if (sensorIndex < 0 || sensorIndex >= sensorCount) {
            Serial.println("Invalid sensor index");
            return 0   ;
        }
        uint16_t distance = sensors[sensorIndex].read();
       // log_i("SENSOR_ %d:" MAGENTA " %d mm" RESET, sensorIndex, distance);
        if (sensors[sensorIndex].timeoutOccurred()) { Serial.print(" TIMEOUT"); }
        return distance;
    }


}