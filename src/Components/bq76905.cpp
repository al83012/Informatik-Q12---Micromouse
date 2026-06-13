#include "Components/BQ76905.h"
#include "Arduino.h"
#include "components/esp32.h"
#include "i2ctool.h"
// Battery Management System (BMS) for 4-16 cells in series



void BQ76905::init() {
    Wire.beginTransmission(BQ76905::ComponentVars::I2C_ADDRESS);
    if (Wire.endTransmission() != 0) {
        log_e("# BQ76905 not found at address 0x08");
    } else {
        log_i("# BQ76905 initialized successfully");
    } 
}

void BQ76905::readAllCellVoltages() {
    BQ76905::CellVoltages::cell1 = getCellVoltage(1);
    BQ76905::CellVoltages::cell2 = getCellVoltage(2);
    BQ76905::CellVoltages::cell3 = getCellVoltage(3);
    BQ76905::CellVoltages::cell4 = getCellVoltage(4);
    BQ76905::CellVoltages::cell5 = getCellVoltage(5);

}

int BQ76905::getCellVoltage(int cell) {
    uint8_t cmd_addr = 0x14 + (cell*2);
    uint8_t voltage;
    I2CTOOL::I2C1Read(BQ76905::ComponentVars::I2C_ADDRESS, cmd_addr, voltage);
    return voltage;

}

void BQ76905::readTemperature() {

}


void BQ76905::alert() {
    //TODO: Handle alert further; For now emergency shutdown
    log_e("# BATTERY MANAGEMENT CRITICAL ALERT! (BQ76905)");
    Esp32::shutdown();
}