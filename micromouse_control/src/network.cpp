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



string Network::getWsUrl() {
    string url = Network::NetworkVars::websockets_server + WiFi.gatewayIP().toString().c_str() + ":" + to_string(Network::NetworkVars::port) + "/";
    return url;
}



void Network::connectWS() {
  string ws_ip = getWsUrl();
  log_i("%s%", ws_ip.c_str());
  Utility::printClient("#Connecting to... " + ws_ip);

  Network::NetworkVars::client = WebsocketsClient();

  Network::NetworkVars::client.onMessage(Handler::handleCommand);
  Network::NetworkVars::client.onEvent(Handler::handleEvent);

  bool connected = Network::NetworkVars::client.connect(ws_ip.c_str());

  if (connected) {
    log_i("# CN SUCC!");
    Utility::debug("From the moment i understood the weakness of my flesh, it disgusted me...");
    if (Master::GlobalVars::reset) {
      Network::NetworkVars::client.send("RESTART");
      Network::NetworkVars::client.send("CONTINUE");
      Master::GlobalVars::reset = false;
    }
    Network::NetworkVars::client.ping();

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
    log_d("%d%", n);
    log_d(" NW FOUND");
    for (int i = 0; i < n; ++i) {
      log_d("%d%", i + 1);
      log_d(": ");
      log_d("" , WiFi.SSID(i));
      log_d(" (");
      log_d("", WiFi.RSSI(i));
      log_d(")");
      log_d("", (WiFi.encryptionType(i) == WIFI_AUTH_OPEN) ? " " : "*");
      delay(10);
    }
  }
  log_d("");
}
void Network::initNetwork() {
 log_d("# INIT NTW CN...");
  WiFi.begin(Network::NetworkVars::ssid, Network::NetworkVars::password);
  log_d("# Connecting");
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    log_d(".");
  }
  log_d("");
  log_i("# SUCC! MY IP: ");
  log_i("", WiFi.localIP());

  WiFi.setAutoReconnect(true);
  WiFi.persistent(true);
  WiFi.setSleep(false);
}
    
void Network::checkNetwork() {
  if (Network::NetworkVars::client.available()) {
    Network::NetworkVars::client.poll();
  } else {
    log_e("# CN LOST!");
    log_d("# RE-CN...");
    Network::connectWS();
    if (Network::NetworkVars::client.available()) {
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

  Network::NetworkVars::client = websockets::WebsocketsClient();

  Network::NetworkVars::client.onMessage(Handler::handleCommand);
  Network::NetworkVars::client.onEvent(Handler::handleEvent);
  Network::connectWS();
}
        