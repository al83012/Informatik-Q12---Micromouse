#include "Arduino.h"
#include "WiFi.h"
#include "ArduinoWebsockets.h"
#include "HTTPClient.h"

#include "utility.h"
#include "master.h"
#include "network.h"
#include "handler.h"
#include "Components/Esp32.h"

void setup() {

  Esp32::initESP32();
  Network::setup();

  Serial.println("# Setup done!");
}

void loop() {
  Network::checkNetwork();
}

