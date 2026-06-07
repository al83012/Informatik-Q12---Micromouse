#include <ArduinoWebsockets.h>
#include <Arduino.h>
#include "WiFi.h"
#include "HTTPClient.h"

#include "master.h"
#include "utility.h"
#include "network.h"

using namespace websockets;
using namespace std;


void Utility::printClient(string message) {
    networkVars.client.send(message.c_str());
}

void Utility::stop() {
    printClient("STOP");
    networkVars.client.send("STOP");
}

void Utility::battery() {
    printClient("BTRY > SRV");
    networkVars.client.send("BATTERY 0");
}

void Utility::restart() {
    printClient("RSTRT > SRV");

    //measurements.clear();
    //reactions.clear();
    globalVars.lastCMD_ID = -1;
    globalVars.currCMD_ID = -1;

    networkVars.client.send("RESTART");
    globalVars.wait_restart_confirm = true;
    Serial.println("# AWAIT RSTRT CONFIRM...");
}
    
void Utility::finishedAll() {
string message = "CMD-FINISHED #";
  Serial.println("# CMD DONE > SRV");
  //Serial.println(String(X));
  // Serial.println(String(Y));
  //Serial.print(globalVars.dir);

  message = message + to_string(globalVars.currCMD_ID);
  printClient(message);
}
     

void Utility::finishedAllInterrupt(string message) {
  string content = "CMD-FINISHED #" + globalVars.currCMD_ID + ' ' + message;
  Serial.println("# CMD DONE INTRPT > SRV");
  printClient(content);
}

void Utility::desync() {
 Serial.println("# DSYNC > SRV");
  string error = "DESYNC ";
  for (int i = globalVars.lastCMD_ID + 1; i < globalVars.currCMD_ID; i++) {
    error = error + "#";
    error = error + std::to_string(i);
    error = error + " ";
  }
  printClient(error);
  Serial.println("# AWAIT RESYNC...");
  globalVars.desync_mode = true;
}

void Utility::debug(string message) {
  Serial.println("# RSTRT > SRV");

  //measurements.clear();
  //reactions.clear();
  globalVars.lastCMD_ID = -1;
  globalVars.currCMD_ID = -1;

  networkVars.client.send(("DBG " + message).c_str());
}

std::vector<string> Utility::splitString(string str, char delimiter) {
    vector<string> tokens;
    string token;
    for (char c : str) {
        if (c == delimiter) {
            if (!token.empty()) {
                tokens.push_back(token);
                token.clear();
            }
        } else {
            token += c;
        }
    }
    if (!token.empty()) {
        tokens.push_back(token);
    }
    return tokens;
}