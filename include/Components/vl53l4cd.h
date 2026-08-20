#ifndef VL53L4CD_H
#define VL53L4CD_H
#include <Wire.h>
#include <VL53L4CD.h>


// Time-of-Flight (ToF) Distance Sensor
namespace TOF {
    
    constexpr uint8_t sensorCount = 3;
   inline VL53L4CD sensors[sensorCount] = {VL53L4CD(), VL53L4CD(), VL53L4CD()};
   inline VL53L4CD sensorFront = sensors[0];
   inline VL53L4CD sensorLeft = sensors[1];
   inline VL53L4CD sensorRight = sensors[2];
    void init();
    void debugReadAllSensors();


}
#endif