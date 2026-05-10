#include <ArduinoWebsockets.h>
#include <Arduino.h>
#include "WiFi.h"
#include "HTTPClient.h"

#include "network.h"
#include "utility.h"
#include "master.h"
#include "handler.h"

using namespace websockets;
using namespace std;

NetworkVars networkVars;


string Network::getWsUrl() {
    auto url = networkVars.websockets_server + to_string(WiFi.gatewayIP()) + to_string(networkVars.port) + "/";
    return url;
}



void Network::connectWS() {
  string ws_ip = getWsUrl();
  Utility::printClient("#Connecting to... " + ws_ip);

  networkVars.client = WebsocketsClient();

  networkVars.client.onMessage(Handler::handleCommand);
  networkVars.client.onEvent(Handler::handleEvent);

  bool connected = networkVars.client.connect(std::string(ws_ip).c_str());

  if (connected) {
    Serial.println("# CN SUCC!");
    Utility::debug("From the moment i understood the weakness of my flesh, it disgusted me...");
    if (globalVars.reset) {
      networkVars.client.send("RESTART");
      networkVars.client.send("CONTINUE");
      globalVars.reset = false;
    }
    networkVars.client.ping();

  } else {
    Serial.println("# CN FAIL!");
  }
}

void Network::scanNetworks() {
  Serial.println("# SCAN FOR NW...");
  int n = WiFi.scanNetworks();

    Serial.println("# SCAN DONE!");
  if (n == 0) {
    Serial.println("# NO NW FOUND.");
  } else {
    Serial.println();
    Serial.print(n);
    Serial.println(" NW FOUND");
    for (int i = 0; i < n; ++i) {
      Serial.print(i + 1);
      Serial.print(": ");
      Serial.print(WiFi.SSID(i));
      Serial.print(" (");
      Serial.print(WiFi.RSSI(i));
      Serial.print(")");
      Serial.println((WiFi.encryptionType(i) == WIFI_AUTH_OPEN) ? " " : "*");
      delay(10);
    }
  }
  Serial.println("");
}
void Network::initNetwork() {
  Serial.println("# INIT NTW CN...");
  WiFi.begin(networkVars.ssid, networkVars.password);
  Serial.println("# Connecting");
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }
  Serial.println("");
  Serial.println("# SUCC! MY IP: ");
  Serial.println(WiFi.localIP());

  WiFi.setAutoReconnect(true);
  WiFi.persistent(true);
  WiFi.setSleep(false);
}
    
        