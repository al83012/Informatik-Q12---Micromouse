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
#include "components/tmp464.h"
#include "measurement.h"
#include "Components/drv8424.h"

#include "i2ctool.h"

void setup() {
delay(1000);
log_i("Initializing Setup");
Esp32::initESP32();
 Network::setup();

 log_i("# Setup done!");
 I2CTOOL::I2CScanner();

 TPL0102::DbgPrintVoltages();
 TMP464::DbgPrintTemperatures(); 
 //Measurement::IR::debugPrintRawDistance(0);

}

void loop() {
 Network::checkNetwork();
 delay(2000);
 log_d(".");
 TMP464::DbgPrintTemperatures();
 Measurement::Sensors::reportTemperature();
 DRV8424::debugPrintEncoderCounts();

}