#ifndef handler
#define handler
#include <String>
#include <vector>
using namespace std;
#include "master.h";

//Command config
const int MAX_CMD_ARGS = 15;
const int MAX_SUB_STEPS = 256;
const int DISTANCE_THRESHOLD = 0;
const int SENSORLIMIT = 5;


//IDs
const int STOP_IF_OPEN_ID = 10;
const int STOP_IF_BLOCKED_ID = 11;
const int CONTINUE_ID = 12;


class Handler {
public:
    static int measure(char dir);
    static void movePassive(int cells);
    static void moveActive(int cells, vector<MeasurementTask>& activeTasks);
};


#endif