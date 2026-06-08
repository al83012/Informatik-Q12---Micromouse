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

void setup() {

  Esp32::initESP32();
  Network::setup();

 log_i("# Setup done!");
  TPL0102::DbgPrintVoltages();
 
}

void loop() {
  Network::checkNetwork();
}

