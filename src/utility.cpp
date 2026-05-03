#include "network.h"
#include <ArduinoWebsockets.h>
#include <Arduino.h>
#include "WiFi.h"
#include "HTTPClient.h"
#include "network.cpp";
#include "master.h";


using namespace websockets;
using namespace std;


void Utility::printClient(string message) {
    client.send(message.c_str());
}

void Utility::stop() {
    printClient("STOP");
    client.send("STOP");
}

void Utility::battery() {
    printClient("BTRY > SRV");
    client.send("BATTERY 0");
}

void Utility::restart() {
    printClient("RSTRT > SRV");

    //measurements.clear();
    //reactions.clear();
    globalVars.lastCMD_ID = -1;
    globalVars.currCMD_ID = -1;

    client.send("RESTART");
}
    
        