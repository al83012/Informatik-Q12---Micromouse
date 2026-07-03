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
#include "Components/drv8424.h"
#include "i2ctool.h"
#include "spitool.h"
#include "measurement.h"

void Esp32::initESP32() {
    Serial.begin(Esp32::HardwareConfig::Serial_Clock);
    initPinStates();
    I2CTOOL::init();
    SPITOOL::init();


    initSubComponents();
   // initInterrupts();

    delay(1000);
    log_i("# ESP32 INIT DONE!");
}

void Esp32::initSubComponents() {
    TCAL6408::init();
    //BQ76905::init();
    TMP464::init();
    //IIS2MDC::init();
    TPL0102::init(3.4, 0.3, 0.1); 
    //Fan::init();
    LSM6DSR::init();   
    
    DRV8424::init(50000);

}

void Esp32::initPinStates() {
    
    pinMode(FAN_EN, OUTPUT);
    pinMode(IRLED_0, OUTPUT);
    pinMode(LSM_INT_1, INPUT);
    pinMode(LSM_INT_2, INPUT);
    pinMode(VL_0_INT, INPUT);
    pinMode(VL_1_INT, INPUT);
    pinMode(VL_2_INT, INPUT);
    pinMode(BQ_INT, INPUT);


    Measurement::IR::init();
}



void Esp32::initInterrupts() {
    attachInterrupt(digitalPinToInterrupt(BQ_INT), BQ76905::alert, FALLING);
    attachInterrupt(digitalPinToInterrupt(TCAL_DRV_INT), TCAL6408::handleInterruptDriver, FALLING);
    
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