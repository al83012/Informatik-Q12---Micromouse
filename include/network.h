#ifndef network
#define network
#include <String>
#include "ArduinoWebsockets.h"
#include "WiFi.h"
#include "Client.h"
using namespace std;


websockets::WebsocketsClient client;

string websockets_server = "ws://";
unsigned long lastReconnectAttempt = 0;
const unsigned long reconnectInterval = 2000;

const char* ssid = "HOTSPOT-TEST";
const char* password = "012345678";
const uint16_t port = 9001;
const char serverName[] = "172.13.1.1";

class Network {
public:
    static string getWsUrl();
    static void connectWS();
    static void initNetwork();
    static void scanNetworks();
};


#endif