#ifndef TMP464_H
#define TMP464_H
#include <Wire.h>
// Temperature Sensor

struct TMP464_ComponentVars {
    const uint8_t I2C_ADDRESS = 0x48; 

    const uint8_t REG_LOCAL_TEMP  = 0x00; // Local Temperature Register
    const uint8_t REG_REMOTE_TEMP = 0x01; // Remote Temperature Register
    const uint8_t REG_CONFIG     = 0x02; // Configuration Register


//Writing in registers may be locked; It seems like we do not need writing for now (doc. p. 17)


};
extern TMP464_ComponentVars tmp464_componentVars;

class TMP464  {
public:
    static void init();
    static float readLocalTemperature();
    static float readRemoteTemperature(uint8_t channel);
    static float convertToCelsius(uint16_t rawValue);
    
};


#endif