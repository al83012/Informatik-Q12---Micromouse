#include "network.h"
#include <ArduinoWebsockets.h>
#include <Arduino.h>
#include "WiFi.h"
#include "HTTPClient.h"
#include "utility.h";
#include "master.h";
#include "handler.h";

using namespace websockets;
using namespace std;

WebsocketsClient client;
String websockets_server = "ws://";
unsigned long lastReconnectAttempt = 0;
const unsigned long reconnectInterval = 2000;

const char* ssid = "HOTSPOT-TEST";
const char* password = "012345678";
const uint16_t port = 9001;
const char serverName[] = "172.13.1.1";



string Network::getWsUrl() {
    return string((websockets_server + WiFi.gatewayIP().toString() + String(port) + "/").c_str());
}



void Network::connectWS() {
  string ws_ip = getWsUrl();
  Utility::printClient("#Connecting to... " + ws_ip);

  client = WebsocketsClient();

  client.onMessage(Handler::handleCommand);
  client.onEvent(handleEvent);

  bool connected = client.connect(std::string(ws_ip).c_str());

  if (connected) {
    Serial.println("# CN SUCC!");
    Utility::debug("From the moment i understood the weakness of my flesh, it disgusted me...");
    if (Master::reset) {
      client.send("RESTART");
      client.send("CONTINUE");
      globalVars.reset = false;
    }
    client.ping();

  } else {
    Serial.println("# CN FAIL!");
  }
}
    
        