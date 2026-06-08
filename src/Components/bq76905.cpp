#include "Components/BQ76905.h"
#include "Arduino.h"
#include "components/esp32.h"
#include "i2ctool.h"
// Battery Management System (BMS) for 4-16 cells in series

BQ76905_ComponentVars bq76905_componentVars;
BQ76905_CellVoltages bq76905_cellVoltages;

void BQ76905::init() {
    Wire.beginTransmission(bq76905_componentVars.I2C_ADDRESS);
    if (Wire.endTransmission() != 0) {
        log_e("# BQ76905 not found at address 0x08");
    } else {
        log_i("# BQ76905 initialized successfully");
    } 
}

void BQ76905::readAllCellVoltages() {
    bq76905_cellVoltages.cell1 = getCellVoltage(1);
    bq76905_cellVoltages.cell2 = getCellVoltage(2);
    bq76905_cellVoltages.cell3 = getCellVoltage(3);
    bq76905_cellVoltages.cell4 = getCellVoltage(4);
    bq76905_cellVoltages.cell5 = getCellVoltage(5);

}

int BQ76905::getCellVoltage(int cell) {
    uint8_t cmd_addr = 0x14 + (cell*2);
    return I2CTOOL::i2c_readRegister16(bq76905_componentVars.I2C_ADDRESS, cmd_addr);

}

void BQ76905::readTemperature() {

}


void BQ76905::alert() {
    //TODO: Handle alert further; For now emergency shutdown
    log_e("# BATTERY MANAGEMENT CRITICAL ALERT! (BQ76905)");
    Esp32::shutdown();
}