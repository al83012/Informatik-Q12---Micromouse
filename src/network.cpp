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
    string url = networkVars.websockets_server + WiFi.gatewayIP().toString().c_str() + ":" + to_string(networkVars.port) + "/";
    return url;
}



void Network::connectWS() {
  string ws_ip = getWsUrl();
  log_i(ws_ip.c_str());
  Utility::printClient("#Connecting to... " + ws_ip);

  networkVars.client = WebsocketsClient();

  networkVars.client.onMessage(Handler::handleCommand);
  networkVars.client.onEvent(Handler::handleEvent);

  bool connected = networkVars.client.connect(ws_ip.c_str());

  if (connected) {
    log_i("# CN SUCC!");
    Utility::debug("From the moment i understood the weakness of my flesh, it disgusted me...");
    if (globalVars.reset) {
      networkVars.client.send("RESTART");
      networkVars.client.send("CONTINUE");
      globalVars.reset = false;
    }
    networkVars.client.ping();

  } else {
    log_e("# CN FAIL!");
  }
}

void Network::scanNetworks() {
  log_i("# SCAN FOR NW...");
  int n = WiFi.scanNetworks();

    log_d("# SCAN DONE!");
  if (n == 0) {
    log_e("# NO NW FOUND.");
  } else {
    log_d();
    log_d(n);
    log_d(" NW FOUND");
    for (int i = 0; i < n; ++i) {
      log_d(i + 1);
      log_d(": ");
      log_d(WiFi.SSID(i));
      log_d(" (");
      log_d(WiFi.RSSI(i));
      log_d(")");
      log_d((WiFi.encryptionType(i) == WIFI_AUTH_OPEN) ? " " : "*");
      delay(10);
    }
  }
  log_d("");
}
void Network::initNetwork() {
 log_d("# INIT NTW CN...");
  WiFi.begin(networkVars.ssid, networkVars.password);
  log_d("# Connecting");
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    log_d(".");
  }
  log_d("");
  log_i("# SUCC! MY IP: ");
  log_i(WiFi.localIP());

  WiFi.setAutoReconnect(true);
  WiFi.persistent(true);
  WiFi.setSleep(false);
}
    
void Network::checkNetwork() {
  if (networkVars.client.available()) {
    networkVars.client.poll();
  } else {
    log_e("# CN LOST!");
    log_d("# RE-CN...");
    Network::connectWS();
    if (networkVars.client.available()) {
    log_d("# RE-CN SUCC!");
    Utility::printClient("CONTINUE");

    }
  }
}

void Network::setup() {
  log_i("# Initializing WiFi...");
  WiFi.mode(WIFI_STA);
  Network::scanNetworks();
  Network::initNetwork();

  networkVars.client = websockets::WebsocketsClient();

  networkVars.client.onMessage(Handler::handleCommand);
  networkVars.client.onEvent(Handler::handleEvent);
  Network::connectWS();
}
        