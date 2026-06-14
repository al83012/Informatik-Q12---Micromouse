#ifndef TMP464_H
#define TMP464_H
#include <Wire.h>
// Temperature Sensor
namespace TMP464 {
namespace ComponentVars {
    constexpr uint8_t I2C_ADDRESS = 0x48; 

    constexpr uint8_t REG_LOCAL_TEMP  = 0x00; // Local Temperature Register
    constexpr uint8_t REG_REMOTE_TEMP = 0x01; // Remote Temperature Register
    //REG_REMOTE_TEMP Registers for other channels not listed here, since they are calculated in methods (REG_REMOTE_TEMP + (channel - 1))
    constexpr uint8_t REG_CONFIG     = 0x02; 

    /*16b Configuration Register 
    [15:12] RESERVED 
    [11:8] REMOTE CH CONVERSIONS 
    [7] LOCAL CH CONVERSIONS 
    [6] ONE SHOT CONV (we prob. do not want this) 
    [5] Shutdown 
    [4:2] Conversion rate 
    [1 BUSY , 0 RESERVED] */ 

    constexpr uint16_t SETTINGS_REG_CONFIG = 0b0000111110011100; // This is also the default reset value

    constexpr uint16_t REG_SOFTWARE_RESET = 0x20;
    constexpr uint16_t REG_THERM_STATUS = 0x21;
    constexpr uint16_t REG_THERM2_STATUS = 0x22;

    //THERM LIMIT RESOLTUION: 0.5 d.C. / LSB -> (+255 to -255)
    constexpr uint16_t REG_LOCAL_THERM_LIMIT = 0x39;
    constexpr uint16_t REG_LOCAL_THERM2_LIMIT = 0x3A;
    constexpr uint16_t REG_REMOTE_1_THERM_LIMIT = 0x42;
    constexpr uint16_t REG_REMOTE_1_THERM2_LIMIT = 0x43;
    constexpr uint16_t REG_REMOTE_2_THERM_LIMIT = 0x4A;
    constexpr uint16_t REG_REMOTE_2_THERM2_LIMIT = 0x4B;
    constexpr uint16_t REG_REMOTE_3_THERM_LIMIT = 0x52;
    constexpr uint16_t REG_REMOTE_3_THERM2_LIMIT = 0x52;
    constexpr uint16_t REG_REMOTE_4_THERM_LIMIT = 0x5A;
    constexpr uint16_t REG_REMOTE_4_THERM2_LIMIT = 0x5B;
//We will probably have 1 or 2 global temp. limits.


//Writing in certain register-bits may be locked; Doesnt affect our code, is for safety reasons / reserved bits.


}
     void init();
     void setStandardConfiguration();

     float readLocalTemperature();
     float readRemoteTemperature(uint8_t channel);

    //10 bit bits available, rest will be ignored automatically (+-255 [0 = -255 | 256 = 0 | 512 = no limit])
     void setLocalTermLimit(uint16_t limit);
     void setLocalTerm2Limit(uint16_t limit);

     void setRemoteTermLimit(uint8_t channel, uint16_t limit);
     void setRemoteTerm2Limit(uint8_t channel, uint16_t limit);

     void setGlobalTermLimits(uint16_t lowestSafeTemp, uint16_t highestSafeTemp);
    
     float convertToCelsius(uint16_t rawValue);

     void setShutdownMode(bool enableShutdown);

     void DbgPrintTemperatures();
     


    


}

#endif