#ifndef MASTER
#define MASTER
#include <String>
using namespace std;

struct MeasurementTask {
  int subStep;
  char direction;
  int reaction;
};

struct GlobalVars {
    bool desync_mode = false;
    int lastCMD_ID = -1;
    int currCMD_ID = -1;
    bool interrupt = false;
    bool reset = true;
} globalVars;


class Master {
};


#endif