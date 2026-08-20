#include "Components/TCAL6408.h"
#include <TCA6408.h>
#include <Components/esp32.h>   
#include "i2ctool.h"

using namespace I2CTOOL;
using namespace TCAL6408;
using namespace Measurement::Sensors;


void TCAL6408::init() {
    // Initialize the TCA6408 I/O expander for the Motor-Driver
    findComponent(SensorNames::TCAL6408_GPIO_EXPANDER_0);

    // Initialize the TCA6408 I/O expander for the Sensors

    findComponent(SensorNames::TCAL6408_GPIO_EXPANDER_1);

    write_init_states();

}

void TCAL6408::write_init_states() {
    // Set initial states for Sensor PCB
    write_register_sensor(ComponentVars::REG_CONFIG, ComponentVars::SENSOR_PCB_INITIAL_STATE); 
    log_d("# TCAL6408: Initial states set for Sensor PCB");

    // Set initial states for Motor Driver PCB
    write_register_driver(ComponentVars::REG_CONFIG, ComponentVars::MOTOR_DRIVER_CB_INITIAL_STATE); 
    log_d("# TCAL6408: Initial states set for Motor Driver PCB");
}

void TCAL6408::setToFToInput() {
    write_register_sensor(ComponentVars::REG_CONFIG, ComponentVars::SENSOR_PCB_TOF_INPUT_STATE); 
    log_d("# TCAL6408: ToF pins set to input mode");
}

uint8_t TCAL6408::read_register_driver(uint8_t reg) {
 uint8_t value;
 I2C1Read(ComponentVars::I2C_ADDRESS_0, reg, value);
  return value;
}

uint8_t TCAL6408::read_register_sensor(uint8_t reg) {
  uint8_t value;
  I2C1Read(ComponentVars::I2C_ADDRESS_1, reg, value);
  return value;
}

void TCAL6408::write_register_driver(uint8_t reg, uint8_t value) {
  I2C1Write(ComponentVars::I2C_ADDRESS_0, reg, value);
}

void TCAL6408::write_register_sensor(uint8_t reg, uint8_t value) {
  I2C1Write(ComponentVars::I2C_ADDRESS_1, reg, value);
}


void TCAL6408::setPinStateSensor(uint8_t pin, bool state) {
    uint8_t currentState = read_register_sensor(ComponentVars::REG_OUTPUT_PORT);
    if (state) {
        currentState |= (1 << pin); // Set bit
    } else {
        currentState &= ~(1 << pin); // Clear bit
    }
    write_register_sensor(ComponentVars::REG_OUTPUT_PORT, currentState);
}

void TCAL6408::setPinStateDriver(uint8_t pin, bool state) {
    uint8_t currentState = read_register_driver(ComponentVars::REG_OUTPUT_PORT);
    if (state) {
        currentState |= (1 << pin); // Set bit
    } else {
        currentState &= ~(1 << pin); // Clear bit
    }
    write_register_driver(ComponentVars::REG_OUTPUT_PORT, currentState);
}

void TCAL6408::shutdownVl53L_0() {
    setPinStateSensor(ComponentVars::PIN_VL53_0_XSHUT, LOW);
}

void TCAL6408::shutdownVl53L_1() {
    setPinStateSensor(ComponentVars::PIN_VL53_1_XSHUT, LOW);
}

void TCAL6408::shutdownVl53L_2() {
    setPinStateSensor(ComponentVars::PIN_VL53_2_XSHUT, LOW);
}

void TCAL6408::setFanRotation(bool state) {
    setPinStateDriver(ComponentVars::PIN_FAN_PH, state);
}

void TCAL6408::setFanAwake(bool state) {
    setPinStateDriver(ComponentVars::PIN_FAN_NSLEEP, state);
}

void TCAL6408::setDriverAwake(bool state) {
    log_i("# (TCAL6408) Setting Driver Awake State to: %s", state ? "Awake" : "Sleep");
    setPinStateDriver(ComponentVars::PIN_DRV_NSLEEP, state);
}

void TCAL6408::handleInterruptDriver() {
    uint8_t intStatus;
    uint8_t pinStates;
    I2C1Read(ComponentVars::I2C_ADDRESS_0, ComponentVars::REG_INT_STATUS, intStatus);
    I2C1Read(ComponentVars::I2C_ADDRESS_0, ComponentVars::REG_INPUT_PORT, pinStates);
    bool fanFaultDetected = (intStatus & (1 << ComponentVars::PIN_FAN_NFAULT)) && !(pinStates & (1 << ComponentVars::PIN_FAN_NFAULT));
    bool thermFaultDetected = ((intStatus & (1 << ComponentVars::PIN_TMP_THERM)) && !(pinStates & (1 << ComponentVars::PIN_TMP_THERM))) || ((intStatus & (1 << ComponentVars::PIN_TMP_THERM2)) && !(pinStates & (1 << ComponentVars::PIN_TMP_THERM2))) ;
   
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
