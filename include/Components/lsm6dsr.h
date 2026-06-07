#ifndef LSM6DSR_H
#define LSM6DSR_H
#include <Wire.h>
#include <SPI.h>

// SPI Communication!!
// 3D- Accelerometer and Gyroscope

//Accelerometer scale mode: 4g (2|4|8|16 possible) (0.122 mg/LSB) -> 16 bit resolution -> Range: -32768 to 32767 -> -4g to +4g
//Gyroscope scale mode: 2000 dps (125|250|500|1000|2000|4000 possible) (70 mdps/LSB) -> 16 bit resolution -> Range: -32768 to 32767 -> -2000 dps to +2000 dps


//TODO: Configure Interrupts!

struct LSM6DSR_ComponentVars {
 const uint8_t REG_WHO_AM_I = 0x0F;
 const uint8_t REG_CTRL1_XL = 0x10; 
 const uint8_t REG_CTRL2_G = 0x11; 
 const uint8_t REG_OUTX_L_G = 0x22;
 const uint8_t REG_OUTX_L_A = 0x28;

};
extern LSM6DSR_ComponentVars lsm6dsr_componentVars;

struct LSM6DSR_Data {
    int16_t accelX;
    int16_t accelY;
    int16_t accelZ;
    int16_t gyroX;
    int16_t gyroY; 
    int16_t gyroZ;
};

class LSM6DSR  {
public:
    static void init();
    static void configureResolution();
    LSM6DSR_Data readSensorData();
    static void readDataBlock(uint8_t startRegister, uint8_t* buffer, size_t length);

    
};



#endif