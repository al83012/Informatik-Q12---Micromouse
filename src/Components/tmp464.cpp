#include "Components/TMP464.h"
#include "Arduino.h"
#include "Components/esp32.h"
#include "i2ctool.h"

// Temperature Sensor
using namespace TMP464;
using namespace I2CTOOL;
using namespace Measurement::Sensors;


void TMP464::init() {
    // Initialize the TMP464 temperature sensor
    findComponent(Measurement::Sensors::SensorNames::TMP464_TEMPERATURE_SENSOR);

    setStandardConfiguration();

}
void TMP464::setStandardConfiguration() {
     log_d("# (TMP464) Trying to set standard configuration");
    I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_CONFIG, ComponentVars::SETTINGS_REG_CONFIG);
}

float TMP464::readLocalTemperature() {
    log_d("# (TMP464) Trying to read local temperature");
    uint16_t rawTemp;
     I2C1Read(ComponentVars::I2C_ADDRESS, ComponentVars::REG_LOCAL_TEMP, rawTemp);
    return convertToCelsius(rawTemp);
}

float TMP464::readRemoteTemperature(uint8_t channel) {
    log_d("# (TMP464) Trying to read remote temperature on channel %d", channel);
    //Channel 1 = 0x01 ; Channel 2 = 0x02 ; Channel 3 = 0x03 ; Channel 4 = 0x04
    uint16_t rawTemp;
    I2C1Read(ComponentVars::I2C_ADDRESS, ComponentVars::REG_REMOTE_TEMP + (channel - 1), rawTemp); 
    return convertToCelsius(rawTemp);
}

float TMP464::convertToCelsius(uint16_t rawValue) {
    log_d("# (TMP464) Converting temperature... %d", rawValue );
    int16_t signedTemp = (int16_t)rawValue; 
    return (signedTemp >> 3) * 0.0625f; //Shift 3 bits, since data is only stored in the upper 13 bits (resolution of 0.0625 - fixed)
}

void TMP464::setLocalTermLimit(uint16_t limit) {
    if(limit > 256) {
    log_e("# Invalid temperature limit (LocalTempLimit - TMP464)!");
    uint16_t formatted_limit = limit << 8;
    } else {
    I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_LOCAL_THERM_LIMIT, limit);
    }
}

void TMP464::setLocalTerm2Limit(uint16_t limit) {
    if(limit > 256) {
    log_e("# Invalid temperature limit2 (LocalTempLimit2 - TMP464)!");
    uint16_t formatted_limit = limit << 8;
    } else {
    I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_LOCAL_THERM2_LIMIT, limit);
    }
}

void TMP464::setRemoteTermLimit(uint8_t channel, uint16_t limit) {
    log_d("# (TMP464) Trying to set RemoteTemp Limit %d", limit , "for channel %d", channel);
    if(limit > 256) {
    log_e("# Invalid temperature limit (RemoteTemp - TMP464)!");
    uint16_t formatted_limit = limit << 8;
    } else {
    
    if(channel == 1) {
        I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_REMOTE_1_THERM_LIMIT, limit);
    } else if(channel == 2) {
        I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_REMOTE_2_THERM_LIMIT, limit);
    } else if(channel == 3) {
        I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_REMOTE_3_THERM_LIMIT, limit);
    } else if(channel == 4) {
        I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_REMOTE_4_THERM_LIMIT, limit);
    } else {
        log_e("# (TMP464) Error while trying to set remote temperature: Invalid channel!");
    }

    }

}

void TMP464::setRemoteTerm2Limit(uint8_t channel, uint16_t limit) {
    log_d("# (TMP464) Trying to set RemoteTemp Limit %d", limit , "for channel %d", channel);
    if(limit > 256) {
    log_e("# Invalid temperature limit (RemoteTemp - TMP464)!");
    uint16_t formatted_limit = limit << 8;
    } else {
    
    if(channel == 1) {
        I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_REMOTE_1_THERM2_LIMIT, limit);
    } else if(channel == 2) {
        I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_REMOTE_2_THERM2_LIMIT, limit);
    } else if(channel == 3) {
        I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_REMOTE_3_THERM2_LIMIT, limit);
    } else if(channel == 4) {
        I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_REMOTE_4_THERM2_LIMIT, limit);
    } else {
        log_e("# (TMP464) Error while trying to set remote temperature: Invalid channel!");
    }

    }
}

void TMP464::setGlobalTermLimits(uint16_t lowestSafeTemp, uint16_t highestSafeTemp) {

    log_d("# (TMP464) Trying to set global temperature limits");

    setLocalTermLimit(lowestSafeTemp);
    setLocalTerm2Limit(highestSafeTemp);

    setRemoteTermLimit(1, lowestSafeTemp);
    setRemoteTerm2Limit(1, highestSafeTemp);

    setRemoteTermLimit(2, lowestSafeTemp);
    setRemoteTerm2Limit(2, highestSafeTemp);

    setRemoteTermLimit(3, lowestSafeTemp);
    setRemoteTerm2Limit(3, highestSafeTemp);

    setRemoteTermLimit(4, lowestSafeTemp);
    setRemoteTerm2Limit(4, highestSafeTemp);

}

void TMP464::setShutdownMode(bool enableShutDown) {
    log_d("# Trying to enable shutdown mode (TMP464)");
    uint16_t current_reg;
    I2C1Read(ComponentVars::I2C_ADDRESS, ComponentVars::REG_CONFIG, current_reg);
    if(enableShutDown) {
        current_reg |= (1 << 5);
        log_d("# Succesfuly enabled shutdown mode! (TMP464) ");
    } else {
        log_d("# Succesfuly disabled shutdown mode! (TMP464) ");
        current_reg &= ~(1 << 5);
    }

    I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_CONFIG, current_reg);

}

void TMP464::DbgPrintTemperatures() {
    log_d("%d", readLocalTemperature());
    log_d("%d", readRemoteTemperature(0x01));
    log_d("%d", readRemoteTemperature(0x02));
    log_d("%d", readRemoteTemperature(0x03));
    log_d("%d", readRemoteTemperature(0x04));
}