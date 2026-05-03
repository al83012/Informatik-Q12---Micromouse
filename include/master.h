#ifndef MASTER
#define MASTER
#include <String>
#include "simulation.h"

using namespace std;
enum directions {
  posX,
  posY,
  negX,
  negY

};

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
    enum directions dir = posX;
} globalVars;


class Master {
};


#endif