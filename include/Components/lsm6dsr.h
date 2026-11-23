#ifndef LSM6DSR_H
#define LSM6DSR_H
#include <Wire.h>
#include <SPI.h>

// SPI Communication!!
// 3D- Accelerometer and Gyroscope
//Accelerometer scale mode: 4g (2|4|8|16 possible) (0.122 mg/LSB) -> 16 bit resolution -> Range: -32768 to 32767 -> -4g to +4g
//Gyroscope scale mode: 2000 dps (125|250|500|1000|2000|4000 possible) (70 mdps/LSB) -> 16 bit resolution -> Range: -32768 to 32767 -> -2000 dps to +2000 dps
//TODO: Configure Interrupts!
namespace LSM6DSR {

namespace ComponentVars {
 constexpr uint8_t REG_WHO_AM_I = 0x0F;
 constexpr uint8_t REG_CTRL1_XL = 0x10; 
 constexpr uint8_t REG_CTRL2_G = 0x11; 
 constexpr uint8_t REG_OUTX_L_G = 0x22;
 constexpr uint8_t REG_OUTX_L_A = 0x28;
extern float gyroBiasZ;
}

struct Data {
    float accelX;
    float accelY;
    float accelZ;
    float gyroX;
    float gyroY;
    float gyroZ;
};


    void init();
    void configureResolution();
    Data readSensorData();
    void readDataBlock(uint8_t startRegister, uint8_t* buffer, size_t length);
    void calibrateGyro(int samples);
    float getAccelX();
    float getAccelY();
    float getAccelZ();

    float getGyroX();
    float getGyroY();
    float getGyroZ();


}


#endif