#include "Components/LSM6DSR.h"
#include "Arduino.h"
#include "Components/esp32.h"
#include "spitool.h"
// Device communication through SPI !!
// 3D- Accelerometer and Gyroscope
LSM6DSR_ComponentVars lsm6dsr_componentVars;
Pins pins = Esp32::pins;

void LSM6DSR::init() {
    uint8_t whoAmI = SPITOOL::spi_readRegister(lsm6dsr_componentVars.REG_WHO_AM_I, pins.PIN_LSM_CS);
    if (whoAmI != 0x6B) {
        log_e("# LSM6DSR (SPI) not found!");
    }
    configureResolution();

    log_i("# LSM6DSR (SPI) initialized successfully");
}

void LSM6DSR::readDataBlock(uint8_t startRegister, uint8_t* buffer, size_t length) {
    for (size_t i = 0; i < length; i++) {
        buffer[i] = SPITOOL::spi_readRegister(startRegister + i, pins.PIN_LSM_CS);
    }
}

LSM6DSR_Data LSM6DSR::readSensorData() {
    uint8_t buffer[12];
    readDataBlock(lsm6dsr_componentVars.REG_OUTX_L_G, buffer, 12);

    LSM6DSR_Data data;
    data.gyroX = (int16_t)(buffer[1] << 8 | buffer[0]);
    data.gyroY = (int16_t)(buffer[3] << 8 | buffer[2]);
    data.gyroZ = (int16_t)(buffer[5] << 8 | buffer[4]);

    data.accelX = (int16_t)(buffer[7] << 8 | buffer[6]);
    data.accelY = (int16_t)(buffer[9] << 8 | buffer[8]);
    data.accelZ = (int16_t)(buffer[11] << 8 | buffer[10]);

    //IMPORTANT: Change conversion factors based on resoltion settings
    //Conversion to degrees per second for gyroscope and g for accelerometer

    data.gyroX = data.gyroX*0.07f;
    data.gyroY = data.gyroY*0.07f;
    data.gyroZ = data.gyroZ*0.07f;

    data.accelX = data.accelX*0.000122f;
    data.accelY = data.accelY*0.000122f;
    data.accelZ = data.accelZ*0.000122f;

    return data;
}

void LSM6DSR::configureResolution() {
    // Configure accelerometer to 4g and gyroscope to 2000 dps
    // TODO: Setup the right refresh rate (Set to 416 Hz for now)

    //Structure (CTRL1_XL) : ODR [7:4] | Scale [3:2] | High Resolution Selection [1] | Must be 0 [0]
    //Structure (CTRL2_G) : ODR [7:4] | Scale [3:2] | Gyro chain full scale ±125 dps [1] | Gyro chain full-scale ±4000 dps [0]


    //Config: +-4g | 2000 dps | 416 Hz ODR
    SPITOOL::spi_writeRegister(lsm6dsr_componentVars.REG_CTRL1_XL, 0b01101000, pins.PIN_LSM_CS); // ODR 416 Hz, 4g
    SPITOOL::spi_writeRegister(lsm6dsr_componentVars.REG_CTRL2_G,  0b01101100, pins.PIN_LSM_CS); // ODR 416 Hz, 2000 dps

    log_d("# LSM6DSR resolution configured: +-4g for accelerometer, 2000 dps for gyroscope");
}