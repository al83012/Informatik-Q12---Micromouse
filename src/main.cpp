#include "Arduino.h"
#include "WiFi.h"
#include "master.h"
#include "network.h"
#include "handler.h"
#include "ArduinoWebsockets.h"
#include "HTTPClient.h"
#include "utility.h"

void setup() {
     Serial.begin(115200);
  Serial.println("# ESP32 boot starting...");


  Serial.println("# Initializing WiFi...");
  WiFi.mode(WIFI_STA);
  Network::scanNetworks();
  Network::initNetwork();

  network::client = websockets::WebsocketsClient();

  network::client.onMessage(Handler::handleCommand);
  network::client.onEvent(Handler::handleEvent);
  Network::connectWS();

  Serial.println("# Setup done!");
}

void loop() {
    
  if (client.available()) {
    client.poll();
  } else {
    Serial.println("# CN LOST!");
    Serial.println("# RE-CN...");
    Network::connectWS();
    if (client.available()) {
    Serial.println("# RE-CN SUCC!");
    Utility::printClient("CONTINUE");

    }
  }

}