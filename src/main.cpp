#include "Arduino.h"
#include "WiFi.h"
#include "ArduinoWebsockets.h"
#include "HTTPClient.h"

#include "utility.h"
#include "master.h"
#include "network.h"
#include "handler.h"

void setup() {
     Serial.begin(115200);
  Serial.println("# ESP32 boot starting...");


  Serial.println("# Initializing WiFi...");
  WiFi.mode(WIFI_STA);
  Network::scanNetworks();
  Network::initNetwork();

  networkVars.client = websockets::WebsocketsClient();

  networkVars.client.onMessage(Handler::handleCommand);
  networkVars.client.onEvent(Handler::handleEvent);
  Network::connectWS();

  Serial.println("# Setup done!");
}

void loop() {
    
  if (networkVars.client.available()) {
    networkVars.client.poll();
  } else {
    Serial.println("# CN LOST!");
    Serial.println("# RE-CN...");
    Network::connectWS();
    if (networkVars.client.available()) {
    Serial.println("# RE-CN SUCC!");
    Utility::printClient("CONTINUE");

    }
  }

}