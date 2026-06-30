#ifndef MASTER_H
#define MASTER_H
#include <String>
#include "simulation.h"

namespace Master {

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

 namespace GlobalVars {
   inline bool desync_mode = false;
   inline bool wait_restart_confirm = false;
   inline int lastCMD_ID = -1;
   inline int currCMD_ID = -1;
   inline bool interrupt = false;
   inline bool reset = true;
   inline enum directions dir = posX;
  }



}

#endif