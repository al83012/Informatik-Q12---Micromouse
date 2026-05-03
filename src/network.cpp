#include "network.h"
#include <ArduinoWebsockets.h>
#include <Arduino.h>
#include "WiFi.h"
#include "HTTPClient.h"
#include "utility.h"
#include "master.h"
#include "handler.h"

using namespace websockets;
using namespace std;




string Network::getWsUrl() {
    auto url = network::websockets_server + to_string(WiFi.gatewayIP()) + to_string(port) + "/";
    return url;
}



void Network::connectWS() {
  string ws_ip = getWsUrl();
  Utility::printClient("#Connecting to... " + ws_ip);

  network::client = WebsocketsClient();

  network::client.onMessage(Handler::handleCommand);
  network::client.onEvent(Handler::handleEvent);

  bool connected = client.connect(std::string(ws_ip).c_str());

  if (connected) {
    Serial.println("# CN SUCC!");
    Utility::debug("From the moment i understood the weakness of my flesh, it disgusted me...");
    if (globalVars.reset) {
      client.send("RESTART");
      client.send("CONTINUE");
      globalVars.reset = false;
    }
    client.ping();

  } else {
    Serial.println("# CN FAIL!");
  }
}
    
        