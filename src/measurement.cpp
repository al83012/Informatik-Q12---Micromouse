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
            case TMP_TEMP_REMOTE_0:    return "TMP_TEMP_REMOTE_0";
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
