#include "measurement.h"
#include <string>
#include "Arduino.h"
#include "utility.h"
#include "Components/bq76905.h"
#include "Components/iis2mdc.h"
#include "Components/tcal6408.h"
#include "Components/tmp464.h"
#include "Components/tpl0102.h"
#include "Components/vl53l4cd.h"
#include "colors.h"

using namespace COLORS;
void Measurement::Sensors::e_sensor(SensorNames SensorName, std::string errorMessage) {
    std::string message = std::string(Measurement::Sensors::to_string(SensorName)) + " " + errorMessage;

    log_e("%s", message.c_str());
}

void Measurement::Sensors::i_sensor(SensorNames SensorName, std::string infoMessage) {
    std::string message = std::string(Measurement::Sensors::to_string(SensorName)) + " " + infoMessage;
    
    log_i("%s", message.c_str());
}

void Measurement::Sensors::d_sensor(SensorNames SensorName, std::string debugMessage) {
    std::string message = std::string(Measurement::Sensors::to_string(SensorName)) + " " + debugMessage;
    
    log_d("%s", message.c_str());
}

void Measurement::Sensors::sendSensorData(SensorData data, float value) {
     log_d("# Sending sensor-data : %s : %f", to_string(data), value); 
     Utility::sensor(to_string(data), value);
}

 const char* Measurement::Sensors::to_string(SensorData val) {
     switch (val) {
            case BATTERY_PERCENT_LEFT: return "BATTERY_PERCENT_LEFT";
            case FAN_SPEED:            return "FAN_SPEED";
            case TOF_0_DISTANCE:       return "TOF_0_DISTANCE";
            case TOF_1_DISTANCE:       return "TOF_1_DISTANCE";
            case TOF_2_DISTANCE:       return "TOF_2_DISTANCE";
            case TMP_TEMP_LOCAL:       return "TMP_TEMP_LOCAL";
            case TMP_TEMP_REMOTE_4:    return "TMP_TEMP_REMOTE_4";
            case TMP_TEMP_REMOTE_1:    return "TMP_TEMP_REMOTE_1";
            case TMP_TEMP_REMOTE_2:    return "TMP_TEMP_REMOTE_2";
            case TMP_TEMP_REMOTE_3:    return "TMP_TEMP_REMOTE_3";
            case POT_WIPER_A:          return "POT_WIPER_A";
            case POT_WIPER_B:          return "POT_WIPER_B";
            case POT_VOLT_A:           return "POT_VOLT_A";
            case POT_VOLT_B:           return "POT_VOLT_B";
            case POT_DEFAULT_VOLT_A:   return "POT_DEFAULT_VOLT_A";
            case POT_DEFAULT_VOLT_B:   return "POT_DEFAULT_VOLT_B";
            case ACC_X:                return "ACC_X";
            case ACC_Y:                return "ACC_Y";
            case ACC_Z:                return "ACC_Z";
            case GYRO_X:               return "GYRO_X";
            case GYRO_Y:               return "GYRO_Y";
            case GYRO_Z:               return "GYRO_Z";
            case IR_0_DISTANCE:        return "IR_0_DISTANCE";
            case IR_1_DISTANCE:        return "IR_1_DISTANCE";
            case IR_2_DISTANCE:        return "IR_2_DISTANCE";
            case IR_3_DISTANCE:        return "IR_3_DISTANCE";
            default:                   return "UNKNOWN";
        }
}

 const char* Measurement::Sensors::to_string(SensorNames val) {
        switch (val) {
            case BQ76905_BATTERY_MANAGEMENT:     return "BQ76905_BATTERY_MANAGEMENT";
            case FAN:                            return "FAN";
            case TCAL6408_GPIO_EXPANDER_0:       return "TCAL6408_GPIO_EXPANDER_0";
            case TCAL6408_GPIO_EXPANDER_1:       return "TCAL6408_GPIO_EXPANDER_1";
            case LSM6DSR_ACCELEROMETER_GYROSKOP: return "LSM6DSR_ACCELEROMETER_GYROSKOP";
            case TMP464_TEMPERATURE_SENSOR:      return "TMP464_TEMPERATURE_SENSOR";
            case TPL0102_POTENTIOMETER:          return "TPL0102_POTENTIOMETER";
            case VL53L4CD_TOF_0:                 return "VL53L4CD_TOF_0";
            case VL53L4CD_TOF_1:                 return "VL53L4CD_TOF_1";
            case VL53L4CD_TOF_2:                 return "VL53L4CD_TOF_2";
            case IIS2MDC_MAGNETOMETER:           return "IIS2MDC_MAGNETOMETER";
            default:                             return "UNKNOWN";
        }
    }


 uint8_t Measurement::Sensors::getI2CAddress(SensorNames Sensor) {
    switch (Sensor) {
            case BQ76905_BATTERY_MANAGEMENT:     return BQ76905::ComponentVars::I2C_ADDRESS;
            case TCAL6408_GPIO_EXPANDER_0:       return TCAL6408::ComponentVars::I2C_ADDRESS_0;
            case TCAL6408_GPIO_EXPANDER_1:       return TCAL6408::ComponentVars::I2C_ADDRESS_1;
            case TMP464_TEMPERATURE_SENSOR:      return TMP464::ComponentVars::I2C_ADDRESS;
            case TPL0102_POTENTIOMETER:          return TPL0102::ComponentVars::I2C_ADDRESS;
            case IIS2MDC_MAGNETOMETER:           return IIS2MDC::ComponentVars::I2C_ADDRESS;
            case VL53L4CD_TOF_0:                 return 0x00;
            case VL53L4CD_TOF_1:                 return 0x00;
            case VL53L4CD_TOF_2:                 return 0x00;
            default:                             return 0x00;
        }
}

void Measurement::IR::Emitters::initIR_LEDs() {
    log_d("# Setting IR_LED_PINS to OUTPUT...");

    for (int i = 0; i < 4; i++)
    {
        pinMode(EMITTERS[i], OUTPUT);
        digitalWrite(EMITTERS[i], LOW);
    }
    
}

void Measurement::IR::Receivers::initPhotoSensors() {
    log_d("# Setting PD_PINS to INPUT...");

    for (int i = 0; i < 4; i++)
    {
        pinMode(RECEIVERS[i], INPUT);
    }
}

int Measurement::IR::Receivers::readAmbientNoise(int channel) {
    log_d("# Trying to read ambient noise on channel %d", channel);

    if(channel >= 4 || channel < 0) {
        log_e(RED "# Invalid channel for reading ambient noise!");
        return 0;
    }
    int noise = analogRead(Measurement::IR::RECEIVERS[channel]);
    return noise;
}

void Measurement::IR::Emitters::enableLED(int channel) {
    log_d("# Trying to enable IR_LED on channel %d", channel);
    if(channel > 4 || channel < 0) {
    log_e(RED "# Invalid channel for pulsing IR_LED!");
    return;
    } else {
    digitalWrite(EMITTERS[channel], HIGH);
    }
}

void Measurement::IR::Emitters::disableLED(int channel) {
    log_d("# Trying to disable IR_LED on channel %d", channel);

    //No print statements or logic for turning off to save time

    digitalWrite(EMITTERS[channel], LOW);

}

int Measurement::IR::Receivers::readDistance(int channel) {
    return analogRead(RECEIVERS[channel]);
}

void Measurement::IR::refreshDistance(int channel) {

    log_d("# Trying to refresh distance on channel %d", channel);
    if(channel > 4 || channel < 0) {
        log_e(RED "# Invalid channel for refreshing distance");
    }

    refreshNoise(channel);
    Measurement::IR::Emitters::enableLED(channel);
    delayMicroseconds(4);
    RawDistances_Unconverted[channel] = Measurement::IR::Receivers::readDistance(channel);
    Measurement::IR::Emitters::disableLED(channel);
    int delta = RawDistances_Unconverted[channel] - Noises[channel];

    if(delta < 0) {
        FinalDistances_Unconverted[channel] = 0;
    } else {
        FinalDistances_Unconverted[channel] = delta;
    }
}

void Measurement::IR::refreshNoise(int channel) {
     log_d("# Refreshing noise on channel %d", channel);

    if(channel > 4 || channel < 0) {
        log_e(RED "# Invalid channel for refreshing ambient noise!");
        return;
    } else {
        Noises[channel] = Measurement::IR::Receivers::readAmbientNoise(channel);
    }
}

int Measurement::IR::getDistance(int channel) {
    if(channel > 4 || channel < 0) {
        return 0 ;
    } else {
        return FinalDistances_Unconverted[channel];
    }
}

int Measurement::IR::getNoise(int channel) {
    if(channel > 4 || channel < 0) {
        return 0 ;
    } else {
        return Noises[channel];
    }
}

void Measurement::IR::debugPrintRawDistance(int channel) {
    log_i("# Noise of channel" CYAN "%d : %d", channel, Noises[channel]);
    log_i("# Raw value of channel" MAGENTA "%d : %d", channel, FinalDistances_Unconverted[channel]);

} 

void Measurement::IR::init() {
    log_d("# Initializing IR-Sensor-System");
    Measurement::IR::Emitters::initIR_LEDs();
    Measurement::IR::Receivers::initPhotoSensors();
}

void Measurement::Sensors::reportTemperature() {

    log_d("# Reporting temperature...");
    Measurement::Sensors::sendSensorData(Measurement::Sensors::SensorData::TMP_TEMP_LOCAL, TMP464::readLocalTemperature());
    Measurement::Sensors::sendSensorData(Measurement::Sensors::SensorData::TMP_TEMP_REMOTE_1, TMP464::readRemoteTemperature(0x01));
    Measurement::Sensors::sendSensorData(Measurement::Sensors::SensorData::TMP_TEMP_REMOTE_2, TMP464::readRemoteTemperature(0x02));
    Measurement::Sensors::sendSensorData(Measurement::Sensors::SensorData::TMP_TEMP_REMOTE_3, TMP464::readRemoteTemperature(0x03));

    
}

void Measurement::IR::calibration::calibrateWallThresholdLeft() {
    log_d("# Calibrating wall threshold for left side...");
    refreshDistance(Measurement::IR::CHANNEL_LEFT);
    int leftDistance = Measurement::IR::getDistance(Measurement::IR::CHANNEL_LEFT);
    log_d("# Left distance:" CYAN " %d", leftDistance);
    Measurement::IR::calibration::wallThresholdLeft = leftDistance; 
}

void Measurement::IR::calibration::calibrateWallThresholdRight() {
    log_d("# Calibrating wall threshold for right side...");
    refreshDistance(Measurement::IR::CHANNEL_RIGHT);
    int rightDistance = Measurement::IR::getDistance(Measurement::IR::CHANNEL_RIGHT);
    log_d("# Right distance:" CYAN " %d", rightDistance);
    Measurement::IR::calibration::wallThresholdRight = rightDistance; 
}

void Measurement::IR::calibration::calibrateWallThresholdFront() {
    log_d("# Calibrating wall threshold for front side...");
    refreshDistance(Measurement::IR::CHANNEL_FRONT1);
    refreshDistance(Measurement::IR::CHANNEL_FRONT2);

    int frontDistance1 = Measurement::IR::getDistance(Measurement::IR::CHANNEL_FRONT1);
    int frontDistance2 = Measurement::IR::getDistance(Measurement::IR::CHANNEL_FRONT2);
    int averageDistance = (frontDistance1 + frontDistance2) / 2;
    log_d("# Front distance:" CYAN " %d", averageDistance);
    Measurement::IR::calibration::wallThresholdFront = averageDistance; 
}

void Measurement::IR::calibration::initCalibration(int calibrationSteps) {
    wallThresholdLeft = 0;
    wallThresholdRight = 0;
    wallThresholdFront = 0;
    absoluteWallThreshold = 0;
    log_d("# Initialized wall thresholds to 0");
    log_i(RED "# CALIBRATION INITIALIZED: Please ensure that the robot is placed in a safe environment for calibration. The robot will measure distances to walls and set thresholds accordingly.");
    log_i(RED "# Calibration will start in 10 seconds...");
    delay(10000);
    log_i(GREEN "# Calibration started...");
    int totalLeft = 0;
    int totalRight = 0;
    int totalFront = 0;

    for(int i = 0; i < calibrationSteps; i++) {
        log_i(RED "# Calibration step" GREEN "%d of %d", i+1, calibrationSteps);
        
        calibrateWallThresholdLeft();
        totalLeft += Measurement::IR::calibration::wallThresholdLeft;
        delay(500);
        calibrateWallThresholdRight();
        totalRight += Measurement::IR::calibration::wallThresholdRight;
        delay(500);
        calibrateWallThresholdFront();
        totalFront += Measurement::IR::calibration::wallThresholdFront;
        delay(500);
    }
    Measurement::IR::calibration::wallThresholdLeft = totalLeft / calibrationSteps;
    Measurement::IR::calibration::wallThresholdRight = totalRight / calibrationSteps;
    Measurement::IR::calibration::wallThresholdFront = totalFront / calibrationSteps;

    log_i(GREEN "# Calibration completed. Wall thresholds set:" RESET);

    log_i(RED "# Left Wall Threshold: " CYAN "%d" RESET, Measurement::IR::calibration::wallThresholdLeft);
    log_i(BLUE "# Right Wall Threshold: " CYAN "%d" RESET, Measurement::IR::calibration::wallThresholdRight);
    log_i(MAGENTA "# Front Wall Threshold: " CYAN "%d" RESET, Measurement::IR ::calibration::wallThresholdFront);
}


bool Measurement::IR::WallDetection::RefreshWallLeft() {
    Measurement::IR::refreshDistance(Measurement::IR::CHANNEL_LEFT);
    int leftDistance = Measurement::IR::getDistance(Measurement::IR::CHANNEL_LEFT);

    if(leftDistance < Measurement::IR::calibration::wallThresholdLeft+(Measurement::IR::calibration::wallThresholdLeft * Measurement::IR::WallDetection::tolerancePercent / 100)) {
        isWallLeft = true;
    } else {
        isWallLeft = false;
    }


    return isWallLeft;
}

bool Measurement::IR::WallDetection::RefreshWallRight() {
    Measurement::IR::refreshDistance(Measurement::IR::CHANNEL_RIGHT);
    int rightDistance = Measurement::IR::getDistance(Measurement::IR::CHANNEL_RIGHT);

    if(rightDistance < Measurement::IR::calibration::wallThresholdRight+(Measurement::IR::calibration::wallThresholdRight * Measurement::IR::WallDetection::tolerancePercent / 100)) {
        isWallRight = true;
    } else {
        isWallRight = false;
    }

    return isWallRight;
}

bool Measurement::IR::WallDetection::RefreshWallFront() {
    Measurement::IR::refreshDistance(Measurement::IR::CHANNEL_FRONT1);
    Measurement::IR::refreshDistance(Measurement::IR::CHANNEL_FRONT2);

    int frontDistance1 = Measurement::IR::getDistance(Measurement::IR::CHANNEL_FRONT1);
    int frontDistance2 = Measurement::IR::getDistance(Measurement::IR::CHANNEL_FRONT2);

    int averageFrontDistance = (frontDistance1 + frontDistance2) / 2;

    if(averageFrontDistance < Measurement::IR::calibration::wallThresholdFront+(Measurement::IR::calibration::wallThresholdFront * Measurement::IR::WallDetection::tolerancePercent / 100)) {
        isWallFront = true;
    } else {
        isWallFront = false;
    }

    return isWallFront;
}

void Measurement::IR::WallDetection::RefreshAllWalls() {
    RefreshWallLeft();
    RefreshWallRight();
    RefreshWallFront();
}


void Measurement::IR::WallDetection::debugPrintWallDetectionStatus() {
    RefreshAllWalls();
    log_i("# Wall Detection Status:");
    log_i("# Left Wall: " CYAN "%s" RESET, isWallLeft ? GREEN "Y" : RED "N");
    log_i("# Right Wall: " CYAN "%s" RESET, isWallRight ? GREEN "Y" : RED "N");
    log_i("# Front Wall: " CYAN "%s" RESET, isWallFront ? GREEN "Y" : RED "N");
}