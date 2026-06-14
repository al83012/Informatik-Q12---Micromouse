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
    Network::NetworkVars::client.send(message.c_str());
}

void Utility::stop() {
    printClient("STOP");
    Network::NetworkVars::client.send("STOP");
}


void Utility::restart() {
    printClient("RSTRT > SRV");

    //measurements.clear();
    //reactions.clear();
    Master::GlobalVars::lastCMD_ID = -1;
    Master::GlobalVars::currCMD_ID = -1;

    Network::NetworkVars::client.send("RESTART");
    Master::GlobalVars::wait_restart_confirm = true;
    log_i("# AWAIT RSTRT CONFIRM...");
}
    
void Utility::finishedAll() {
string message = "CMD-FINISHED #";
  log_d("# CMD DONE > SRV");
  //Serial.println(String(X));
  // Serial.println(String(Y));
  //Serial.print(Master::GlobalVars::dir);

  message = message + to_string(Master::GlobalVars::currCMD_ID);
  printClient(message);
}
     

void Utility::finishedAllInterrupt(string message) {
  string content = "CMD-FINISHED #" + Master::GlobalVars::currCMD_ID + ' ' + message;
  log_d("# CMD DONE INTRPT > SRV");
  printClient(content);
}

void Utility::desync() {
 log_e("# DSYNC > SRV");
  string error = "DESYNC ";
  for (int i = Master::GlobalVars::lastCMD_ID + 1; i < Master::GlobalVars::currCMD_ID; i++) {
    error = error + "#";
    error = error + std::to_string(i);
    error = error + " ";
  }
  printClient(error);
  log_e("# AWAIT RESYNC...");
  Master::GlobalVars::desync_mode = true;
}

void Utility::debug(string message) {
  log_d("# RSTRT > SRV");

  //measurements.clear();
  //reactions.clear();
  Master::GlobalVars::lastCMD_ID = -1;
  Master::GlobalVars::currCMD_ID = -1;

  Network::NetworkVars::client.send(("DBG " + message).c_str());
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

void Utility::sensor(string name, float value) {
    log_i("# SENSOR > SRV");
    printClient("SENSOR " + name + " " + to_string(value));

}