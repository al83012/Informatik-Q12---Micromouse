#ifndef HANDLER_H
#define HANDLER_H

#include <String>
#include <vector>
using namespace std;
#include "master.h"
#include "ArduinoWebsockets.h"
#include "WiFi.h"
#include "HTTPClient.h"

namespace Handler {

namespace HandleVars {
//Command config
constexpr int MAX_CMD_ARGS = 15;
constexpr int MAX_SUB_STEPS = 256;
constexpr int DISTANCE_THRESHOLD = 0;
constexpr int SENSORLIMIT = 5;


//IDs
constexpr int STOP_IF_OPEN_ID = 10;
constexpr int STOP_IF_BLOCKED_ID = 11;
constexpr int CONTINUE_ID = 12;

} 


 void handleEvent(websockets::WebsocketsEvent event, String data);
 void handleCommand(websockets::WebsocketsMessage WSmessage);
 int measure(char dir);
 void movePassive(int cells);
 void moveActive(int cells, vector<Master::MeasurementTask>& activeTasks);
 void turnPassive(int turns);
 void turnActive(int turns, vector<Master::MeasurementTask>& activeTasks);


}
#endif