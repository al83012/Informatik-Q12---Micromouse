#ifndef TCAL6408_H
#define TCAL6408_H
#include <Wire.h>

// I/O Expander
namespace TCAL6408 {
namespace ComponentVars {

    //Adresses (2 Sperate expanders on pcb)
    constexpr uint8_t I2C_ADDRESS_0 = 0x20; //MOTOR-DRIVER_CB -> SDA/SCL 1
    constexpr uint8_t I2C_ADDRESS_1 = 0x21; //SENSOR_PCB -> SDA/SCL 1

    //Register Map
    constexpr uint8_t REG_INPUT_PORT = 0x00; //Read state of input pins
    constexpr uint8_t REG_OUTPUT_PORT = 0x01; //Write state of output pins
    constexpr uint8_t REG_POLARITY_INV = 0x02; //Invert polarity of input pins (1 = inverted)
    constexpr uint8_t REG_CONFIG = 0x03; // Set PX to 1 = Input (Default) or  0 = Output
    constexpr uint8_t REG_INT_STATUS = 0x46;

    //Agile I/O Specific Registers (Do not know if they'll be used)

    //TODO: Implement registers

    //SENSOR_PCB Pin Assignments
    constexpr uint8_t PIN_VL53_0_XSHUT = 5; // O
    constexpr uint8_t PIN_VL53_1_XSHUT = 6; // O
    constexpr uint8_t PIN_VL53_2_XSHUT = 7; // O

    //SENSOR_PCB Initial Pin States
    constexpr uint8_t SENSOR_PCB_INITIAL_STATE = 0b00011111;

    //MOTOR-DRIVER_CB  Pin Assignments
    constexpr uint8_t PIN_TMP_THERM = 0; // I
    constexpr uint8_t PIN_TMP_THERM2 = 1; // I

    constexpr uint8_t PIN_FAN_PH = 2; // O
    constexpr uint8_t PIN_FAN_NFAULT = 3; // I
    constexpr uint8_t PIN_FAN_NSLEEP = 4; // O
    constexpr uint8_t PIN_DRV_NSLEEP = 6; // O
    constexpr uint8_t PIN_DRV_NFAULT = 7; // I

    //MOTOR-DRIVER_CB Initial Pin States
    constexpr uint8_t MOTOR_DRIVER_CB_INITIAL_STATE = 0b10101011; 
}


     void init();
     void write_init_states();

     uint8_t read_register_sensor(uint8_t registerAdress);
     uint8_t read_register_driver(uint8_t registerAdress);
    
     void write_register_sensor(uint8_t registerAdress, uint8_t value);
     void write_register_driver(uint8_t registerAdress, uint8_t value);

    void setPinStateSensor(uint8_t pin, bool state);
    void setPinStateDriver(uint8_t pin, bool state);

    void shutdownVl53L_0();
    void shutdownVl53L_1();
    void shutdownVl53L_2();

    void setFanRotation(bool state);  // HIGH = Clockwise, LOW = Counterclockwise
    void setFanAwake(bool state);  // HIGH = Awake, LOW = Sleep
    void setDriverAwake(bool state);  // HIGH = Awake, LOW = Sleep

    void handleInterruptDriver();
    void handleInterruptSensor();

}


#endif