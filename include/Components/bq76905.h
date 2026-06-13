#ifndef BQ76905_H
#define BQ76905_H
#include <Wire.h>
// Battery Management System (BMS) for 2-5 cells in series
// Voltage values are stored in mV
namespace BQ76905
{
    
namespace ComponentVars {
    constexpr uint8_t I2C_ADDRESS = 0x08; 
}

namespace CellVoltages {
   inline float cell1;
   inline float cell2;
   inline float cell3;
   inline float cell4;
   inline float cell5;
}


namespace Temperature {
    inline int temperature;  
};

    void init();
    void readAllCellVoltages();
    int getCellVoltage(int Cell);
    void readTemperature(); 
    void alert();

} 
#endif