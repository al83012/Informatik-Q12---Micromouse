#ifndef MC_NETWORK_H
#define MC_NETWORK_H
#include <String>
#include "ArduinoWebsockets.h"
#include "WiFi.h"
#include "Client.h"
using namespace std;

namespace Network {

namespace NetworkVars {
inline websockets::WebsocketsClient client;

inline string websockets_server = "ws://";
inline unsigned long lastReconnectAttempt = 0;
constexpr unsigned long reconnectInterval = 2000;
constexpr const char* ssid = "HOTSPOT-TEST";
constexpr const char* password = "012345678";
constexpr uint16_t port = 9001;
constexpr const char* serverName = "172.13.1.1";
}


    string getWsUrl();
    void connectWS();
    void initNetwork();
    void scanNetworks();
    void checkNetwork();
    void setup();
    


}

#endif