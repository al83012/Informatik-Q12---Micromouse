#include "Arduino.h"
#include "WiFi.h"
#include "ArduinoWebsockets.h"
#include "HTTPClient.h"

#include "utility.h"
#include "master.h"
#include "network.h"
#include "handler.h"
#include "Components/Esp32.h"
#include "components/tpl0102.h"
#include "i2ctool.h"

void setup() {
delay(1000);
log_i("Initializing Setup");
Esp32::initESP32();
 // Network::setup();

 log_i("# Setup done!");
 I2CTOOL::I2CScanner();

 TPL0102::DbgPrintVoltages();
 
}

void loop() {
 // Network::checkNetwork();
 delay(200);
 log_d(".");
}