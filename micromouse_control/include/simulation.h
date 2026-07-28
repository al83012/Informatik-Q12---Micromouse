#ifndef SIMULATION_H
#define SIMULATION_H
#include <String>
using namespace std;

namespace Simulation {

namespace SimulationVars {
constexpr int SIM_SIZE = 8;
constexpr int cellSize = 18;
constexpr int stepFreq = 1;
inline int X = 0;
inline int Y = 0;
constexpr int poX = 1;
constexpr int poY = 4;
constexpr int neX = 8;
constexpr int neY = 2;
constexpr int emp = 0;


inline int SIM_FIELD[8][8] = {
  { neY + neX, neY, neY, neY, neY, neY, neY, neY + poX },
  { neX, emp, poX, neX, emp, emp, emp, poX },
  { neX, emp, emp, emp, emp, emp, emp, poX },
  { neX, emp, poX, neX, emp, emp, emp, poX },
  { neX, emp, poX, neX, emp, emp, emp, poX },
  { neX, emp, poX, neX, emp, emp, emp, poX },
  { neX, emp, poX, neX, emp, emp, emp, poX },
  { neX + poY, poY, poY, poY, poY, poY, poY, poX + poY }
};

}

    void sim_move(int n);
    void sim_turn(int turns);
    int sim_measure(char scan_dir);
    bool can_move(int X, int Y, int direction_flag);

}

#endif