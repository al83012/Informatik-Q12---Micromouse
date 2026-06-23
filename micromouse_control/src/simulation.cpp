#include "Arduino.h"

#include "simulation.h"
#include "utility.h"
#include "master.h"
#include "handler.h"


bool Simulation::can_move(int x, int y, int direction_flag) {
if (Simulation::SimulationVars::SIM_FIELD[y][x] & direction_flag) {
    return false;
  }
  return true;
}


void Simulation::sim_move(int n) {
  int newX = Simulation::SimulationVars::X;
  int newY = Simulation::SimulationVars::Y;
  int dir_flag = 0;
  if (Master::GlobalVars::dir == Master::directions::posX) {
    newX += 1;
    dir_flag = 1;
  } else if (Master::GlobalVars::dir == Master::directions::posY) {
    newY += 1;
    dir_flag = 4;
  } else if (Master::GlobalVars::dir == Master::directions::negX) {
    newX -= 1;
    dir_flag = 8;
  } else if (Master::GlobalVars::dir == Master::directions::negY) {
    newY -= 1;
    dir_flag = 2;
  } 
   
  

  if (can_move(newX, newY, dir_flag) && newX < Simulation::SimulationVars::SIM_SIZE && newY < Simulation::SimulationVars::SIM_SIZE) {
    Simulation::SimulationVars::X = newX;
    Simulation::SimulationVars::Y = newY;
  } else {
    log_e("# CANNOT MOVE");
  }
}
void Simulation::sim_turn(int turns) {
if (turns < 0) {
    turns = turns * -1;
    for (int i = 1; i <= turns; i++) {
      if (Master::GlobalVars::dir == Master::directions::posX) {
        Master::GlobalVars::dir = Master::directions::posY;
      } else if (Master::GlobalVars::dir == Master::directions::posY) {
        Master::GlobalVars::dir = Master::directions::negX;
      } else if (Master::GlobalVars::dir == Master::directions::negX) {
        Master::GlobalVars::dir = Master::directions::negY;
      } else if (Master::GlobalVars::dir == Master::directions::negY) {
        Master::GlobalVars::dir = Master::directions::posX;
      }
    }

  } else {
    for (int i = 1; i <= turns; i++) {
      if (Master::GlobalVars::dir == Master::directions::posX) {
        Master::GlobalVars::dir = Master::directions::negY;
      } else if (Master::GlobalVars::dir == Master::directions::negY) {
        Master::GlobalVars::dir = Master::directions::negX;
      } else if (Master::GlobalVars::dir == Master::directions::negX) {
        Master::GlobalVars::dir = Master::directions::posY;
      } else if (Master::GlobalVars::dir == Master::directions::posY) {
        Master::GlobalVars::dir = Master::directions::posX;
      }
    }
  }
}


int Simulation::sim_measure(char scan_dir) {
  int distance = 0;
  int check_X = Simulation::SimulationVars::X;
  int check_Y = Simulation::SimulationVars::Y;
  Master::directions lookDir = Master::GlobalVars::dir;



  if (scan_dir == 'L') {
    if (Master::GlobalVars::dir == Master::directions::posX) lookDir = Master::directions::negY;
    else if (Master::GlobalVars::dir == Master::directions::posY) lookDir = Master::directions::posX;
    else if (Master::GlobalVars::dir == Master::directions::negX) lookDir = Master::directions::posY;
    else if (Master::GlobalVars::dir == Master::directions::negY) lookDir = Master::directions::negX;
  }

  else if (scan_dir == 'R') {
    if (Master::GlobalVars::dir == Master::directions::posX) lookDir = Master::directions::posY;
    else if (Master::GlobalVars::dir == Master::directions::negY) lookDir = Master::directions::posX;
    else if (Master::GlobalVars::dir == Master::directions::negX) lookDir = Master::directions::negY;
    else if (Master::GlobalVars::dir == Master::directions::posY) lookDir = Master::directions::negX;
  }

  int wall_flag = 0;

  if (lookDir == Master::directions::posX) {
    wall_flag = Simulation::SimulationVars::poX;
  } else if (lookDir == Master::directions::posY) {
    wall_flag = Simulation::SimulationVars::poY;
  } else if (lookDir == Master::directions::negX) {
    wall_flag = Simulation::SimulationVars::neX;
  } else if (lookDir == Master::directions::negY) {
    wall_flag = Simulation::SimulationVars::neY;
  }

  while (true) {

    int next_X = check_X;
    int next_Y = check_Y;


    if (distance >= Simulation::SimulationVars::SIM_SIZE) break;
    if (Simulation::SimulationVars::SIM_FIELD[check_Y][check_X] & wall_flag) { break; }

    if (lookDir == Master::directions::posX) {
      next_X++;
    } else if (lookDir == Master::directions::posY) {
      next_Y++;
    } else if (lookDir == Master::directions::negX) {
      next_X--;
    } else if (lookDir == Master::directions::negY) {
      next_Y--;
    }

    if (next_X < 0 || next_X >= Simulation::SimulationVars::SIM_SIZE || next_Y < 0 || next_Y >= Simulation::SimulationVars::SIM_SIZE) { break; }

    check_X = next_X;
    check_Y = next_Y;
    distance++;
  }

  return distance;
}

