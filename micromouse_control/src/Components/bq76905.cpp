#include "Components/BQ76905.h"
#include "Arduino.h"
#include "components/esp32.h"
#include "i2ctool.h"
// Battery Management System (BMS) for 4-16 cells in series
using namespace Measurement::Sensors;
using namespace BQ76905;


void BQ76905::init() {
    I2CTOOL::findComponent(SensorNames::BQ76905_BATTERY_MANAGEMENT);
}

void BQ76905::readAllCellVoltages() {
    CellVoltages::cell1 = getCellVoltage(1);
    CellVoltages::cell2 = getCellVoltage(2);
    CellVoltages::cell3 = getCellVoltage(3);
    CellVoltages::cell4 = getCellVoltage(4);
    CellVoltages::cell5 = getCellVoltage(5);

}

int BQ76905::getCellVoltage(int cell) {
    uint8_t cmd_addr = 0x14 + (cell*2);
    uint8_t voltage;
    I2CTOOL::I2C1Read(ComponentVars::I2C_ADDRESS, cmd_addr, voltage);
    return voltage;

}

void BQ76905::readTemperature() {

}


void BQ76905::alert() {
    //TODO: Handle alert further; For now emergency shutdown
    e_sensor(SensorNames::BQ76905_BATTERY_MANAGEMENT, "CRITICAL ALERT!");

    Esp32::shutdown();
}