#ifndef BQ76905_H
#define BQ76905_H
#include <Wire.h>
// Battery Management System (BMS) for 2-5 cells in series
// Voltage values are stored in mV

struct BQ76905_ComponentVars {
    const uint8_t I2C_ADDRESS = 0x08; 
};
extern BQ76905_ComponentVars bq76905_componentVars;

struct BQ76905_CellVoltages {
    float cell1;
    float cell2;
    float cell3;
    float cell4;
    float cell5;
};

extern BQ76905_CellVoltages cellVoltages;

struct BQ76905_Temperature {
    int temperature;  
};
extern BQ76905_Temperature bq76905_temperature;

class BQ76905 {
public:
    static void init();
    static void readAllCellVoltages();
    static int getCellVoltage(int Cell);
    static void readTemperature(); 
    static void alert();
};

#endif