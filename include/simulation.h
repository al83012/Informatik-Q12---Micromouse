#ifndef simulation
#define simulation
#include <String>
using namespace std;

 struct SimulationVars {
const int SIM_SIZE = 8;
int cellSize = 18;
int stepFreq = 1;
int X = 0;
int Y = 0;
const int poX = 1;
const int poY = 4;
const int neX = 8;
const int neY = 2;
const int emp = 0;


int SIM_FIELD[8][8] = {
  { neY + neX, neY, neY, neY, neY, neY, neY, neY + poX },
  { neX, emp, poX, neX, emp, emp, emp, poX },
  { neX, emp, emp, emp, emp, emp, emp, poX },
  { neX, emp, poX, neX, emp, emp, emp, poX },
  { neX, emp, poX, neX, emp, emp, emp, poX },
  { neX, emp, poX, neX, emp, emp, emp, poX },
  { neX, emp, poX, neX, emp, emp, emp, poX },
  { neX + poY, poY, poY, poY, poY, poY, poY, poX + poY }
};

};
extern SimulationVars simulationVars;




class Simulation {
public:
    static void sim_move(int n);
    static void sim_turn(int turns);
    static int sim_measure(char scan_dir);
    static bool can_move(int X, int Y, int direction_flag);
};


#endif