#include "Arduino.h"
#include "Wire.h"
#include "WiFi.h"
#include "components/esp32.h"
#include <SPI.h>
#include "Components/lsm6dsr.h"
#include "Components/TCAL6408.h"
#include "Components/IIS2MDC.h"
#include "Components/BQ76905.h"
#include "Components/vl53l4cd.h"
#include "Components/tpl0102.h" 
#include "Components/tmp464.h"
#include "Components/fan.h"
#include "Components/iis2mdc.h"
#include "i2ctool.h"
#include "spitool.h"

Pins Esp32::pins;
HardwareConfig Esp32::hardwareConfig;

void Esp32::initESP32() {
    Serial.begin(hardwareConfig.Serial_Clock);
    initPinStates();
    I2CTOOL::init();
    SPITOOL::init();


    initSubComponents();
    initInterrupts();

    delay(1000);
    log_i("# ESP32 INIT DONE!");
}

void Esp32::initSubComponents() {
    TCAL6408::init();
    LSM6DSR::init();
    BQ76905::init();
    TMP464::init();
    IIS2MDC::init();
    TPL0102::init(3.2); // TODO: Conf. highVoltage
    Fan::init();
}

void Esp32::initPinStates() {
    
    pinMode(pins.PIN_FAN_EN, OUTPUT);
    pinMode(pins.PIN_IR_LED_0, OUTPUT);
    pinMode(pins.PIN_IR_LED_1, OUTPUT);
    pinMode(pins.PIN_IR_LED_2, OUTPUT);
    pinMode(pins.PIN_IR_LED_3, OUTPUT);
    pinMode(pins.PIN_LSM_INT_1, INPUT);
    pinMode(pins.PIN_LSM_INT_2, INPUT);
    pinMode(pins.PIN_VL53_0_INT, INPUT);
    pinMode(pins.PIN_VL53_1_INT, INPUT);
    pinMode(pins.PIN_VL53_2_INT, INPUT);
    pinMode(pins.PIN_BQ_ALERT, INPUT);

}



void Esp32::initInterrupts() {
    attachInterrupt(digitalPinToInterrupt(pins.PIN_BQ_ALERT), BQ76905::alert, FALLING);
    attachInterrupt(digitalPinToInterrupt(pins.PIN_TCCAL_DRV_INT), TCAL6408::handleInterruptDriver, FALLING);
    
}

void Esp32::shutdown() {
    log_i("# SHUTTING DOWN ESP32");
    Serial.flush();

    WiFi.disconnect(true);
    WiFi.mode(WIFI_OFF);
    log_i("# ENTERING DEEP-SLEEP");
    delay(100);
    esp_deep_sleep_start();
}