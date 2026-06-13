#include "Components/TCAL6408.h"
#include <TCA6408.h>
#include <Components/esp32.h>   
#include "i2ctool.h"

void TCAL6408::init() {
    // Initialize the TCA6408 I/O expander for the Motor-Driver
    Wire.beginTransmission(TCAL6408::ComponentVars::I2C_ADDRESS_0);
    if (Wire.endTransmission() != 0) {
        log_e("# TCAL6408 (Motor Driver) not found at address 0b0100000");
    } else {
        log_i("# TCAL6408 (Motor Driver) initialized successfully");
    }

    // Initialize the TCA6408 I/O expander for the Sensors

    Wire.beginTransmission(TCAL6408::ComponentVars::I2C_ADDRESS_1);
    if (Wire.endTransmission() != 0) {
        log_e("# TCAL6408 (Sensor) not found at address 0b0100000");
    } else {
        log_i("# TCAL6408 (Sensor) initialized successfully");
    }

    write_init_states();

}

void TCAL6408::write_init_states() {
    // Set initial states for Sensor PCB
    write_register_sensor(TCAL6408::ComponentVars::REG_CONFIG, TCAL6408::ComponentVars::SENSOR_PCB_INITIAL_STATE); 
    log_d("# TCAL6408: Initial states set for Sensor PCB");

    // Set initial states for Motor Driver PCB
    write_register_driver(TCAL6408::ComponentVars::REG_CONFIG, TCAL6408::ComponentVars::MOTOR_DRIVER_CB_INITIAL_STATE); 
    log_d("# TCAL6408: Initial states set for Motor Driver PCB");
}

uint8_t TCAL6408::read_register_driver(uint8_t reg) {
 uint8_t value;
 I2CTOOL::I2C1Read(TCAL6408::ComponentVars::I2C_ADDRESS_0, reg, value);
  return value;
}

uint8_t TCAL6408::read_register_sensor(uint8_t reg) {
  uint8_t value;
  I2CTOOL::I2C1Read(TCAL6408::ComponentVars::I2C_ADDRESS_1, reg, value);
  return value;
}

void TCAL6408::write_register_driver(uint8_t reg, uint8_t value) {
  I2CTOOL::I2C1Write(TCAL6408::ComponentVars::I2C_ADDRESS_0, reg, value);
}

void TCAL6408::write_register_sensor(uint8_t reg, uint8_t value) {
  I2CTOOL::I2C1Write(TCAL6408::ComponentVars::I2C_ADDRESS_1, reg, value);
}


void TCAL6408::setPinStateSensor(uint8_t pin, bool state) {
    uint8_t currentState = read_register_sensor(TCAL6408::ComponentVars::REG_OUTPUT_PORT);
    if (state) {
        currentState |= (1 << pin); // Set bit
    } else {
        currentState &= ~(1 << pin); // Clear bit
    }
    write_register_sensor(TCAL6408::ComponentVars::REG_OUTPUT_PORT, currentState);
}

void TCAL6408::setPinStateDriver(uint8_t pin, bool state) {
    uint8_t currentState = read_register_driver(TCAL6408::ComponentVars::REG_OUTPUT_PORT);
    if (state) {
        currentState |= (1 << pin); // Set bit
    } else {
        currentState &= ~(1 << pin); // Clear bit
    }
    write_register_driver(TCAL6408::ComponentVars::REG_OUTPUT_PORT, currentState);
}

void TCAL6408::shutdownVl53L_0() {
    setPinStateSensor(TCAL6408::ComponentVars::PIN_VL53_0_XSHUT, LOW);
}

void TCAL6408::shutdownVl53L_1() {
    setPinStateSensor(TCAL6408::ComponentVars::PIN_VL53_1_XSHUT, LOW);
}

void TCAL6408::shutdownVl53L_2() {
    setPinStateSensor(TCAL6408::ComponentVars::PIN_VL53_2_XSHUT, LOW);
}

void TCAL6408::setFanRotation(bool state) {
    setPinStateDriver(TCAL6408::ComponentVars::PIN_FAN_PH, state);
}

void TCAL6408::setFanAwake(bool state) {
    setPinStateDriver(TCAL6408::ComponentVars::PIN_FAN_NSLEEP, state);
}

void TCAL6408::setDriverAwake(bool state) {
    setPinStateDriver(TCAL6408::ComponentVars::PIN_DRV_NSLEEP, state);
}

void TCAL6408::handleInterruptDriver() {
    uint8_t intStatus;
    uint8_t pinStates;
    I2CTOOL::I2C1Read(TCAL6408::ComponentVars::I2C_ADDRESS_0, TCAL6408::ComponentVars::REG_INT_STATUS, intStatus);
    I2CTOOL::I2C1Read(TCAL6408::ComponentVars::I2C_ADDRESS_0, TCAL6408::ComponentVars::REG_INPUT_PORT, pinStates);
    bool fanFaultDetected = (intStatus & (1 << TCAL6408::ComponentVars::PIN_FAN_NFAULT)) && !(pinStates & (1 << TCAL6408::ComponentVars::PIN_FAN_NFAULT));
    bool thermFaultDetected = ((intStatus & (1 << TCAL6408::ComponentVars::PIN_TMP_THERM)) && !(pinStates & (1 << TCAL6408::ComponentVars::PIN_TMP_THERM))) || ((intStatus & (1 << TCAL6408::ComponentVars::PIN_TMP_THERM2)) && !(pinStates & (1 << TCAL6408::ComponentVars::PIN_TMP_THERM2))) ;
   
    if (fanFaultDetected) {
            log_i("# FAN EMERGENCY (TCAL6408/Driver)!");
            // Safely regulate fan; For now well shutdown
            Esp32::shutdown();
        }

    if (thermFaultDetected) {
            log_i("# TEMP EMERGENCY (TCAL6408/Driver)!");
            // Emergency shutdown for now; maybe add emergency cooling later on
            Esp32::shutdown();
        }

}
