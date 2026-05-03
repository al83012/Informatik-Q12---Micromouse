#include "simulation.h"
#include "utility.h"
#include "Arduino.h"
#include "master.h"
#include "handler.h"


bool Simulation::can_move(int x, int y, int direction_flag) {
if (SIM_FIELD[y][x] & direction_flag) {
    return false;
  }
  return true;
}


void Simulation::sim_move(int n) {
  int newX = X;
  int newY = Y;
  int dir_flag = 0;
  if (globalVars.dir == posX) {
    newX += 1;
    dir_flag = 1;
  } else if (globalVars.dir == posY) {
    newY += 1;
    dir_flag = 4;
  } else if (globalVars.dir == negX) {
    newX -= 1;
    dir_flag = 8;
  } else if (globalVars.dir == negY) {
    newY -= 1;
    dir_flag = 2;
  } 
   
  

  if (can_move(newX, newY, dir_flag) && newX < SIM_SIZE && newY < SIM_SIZE) {
    X = newX;
    Y = newY;
  } else {
    Serial.println("# CANNOT MOVE");
  }
}
void Simulation::sim_turn(int turns) {
if (turns < 0) {
    turns = turns * -1;
    for (int i = 1; i <= turns; i++) {
      if (globalVars.dir == posX) {
        globalVars.dir = posY;
      } else if (globalVars.dir == posY) {
        globalVars.dir = negX;
      } else if (globalVars.dir == negX) {
        globalVars.dir = negY;
      } else if (globalVars.dir == negY) {
        globalVars.dir = posX;
      }
    }

  } else {
    for (int i = 1; i <= turns; i++) {
      if (globalVars.dir == posX) {
        globalVars.dir = negY;
      } else if (globalVars.dir == negY) {
        globalVars.dir = negX;
      } else if (globalVars.dir == negX) {
        globalVars.dir = posY;
      } else if (globalVars.dir == posY) {
        globalVars.dir = posX;
      }
    }
  }
}


int Simulation::sim_measure(char scan_dir) {
  int distance = 0;
  int check_X = X;
  int check_Y = Y;
  directions lookDir = globalVars.dir;



  if (scan_dir == 'L') {
    if (globalVars.dir == posX) lookDir = negY;
    else if (globalVars.dir == posY) lookDir = posX;
    else if (globalVars.dir == negX) lookDir = posY;
    else if (globalVars.dir == negY) lookDir = negX;
  }

  else if (scan_dir == 'R') {
    if (globalVars.dir == posX) lookDir = posY;
    else if (globalVars.dir == negY) lookDir = posX;
    else if (globalVars.dir == negX) lookDir = negY;
    else if (globalVars.dir == posY) lookDir = negX;
  }

  int wall_flag = 0;

  if (lookDir == posX) {
    wall_flag = poX;
  } else if (lookDir == posY) {
    wall_flag = poY;
  } else if (lookDir == negX) {
    wall_flag = neX;
  } else if (lookDir == negY) {
    wall_flag = neY;
  }

  while (true) {

    int next_X = check_X;
    int next_Y = check_Y;


    if (distance >= SIM_SIZE) break;
    if (SIM_FIELD[check_Y][check_X] & wall_flag) { break; }

    if (lookDir == posX) {
      next_X++;
    } else if (lookDir == posY) {
      next_Y++;
    } else if (lookDir == negX) {
      next_X--;
    } else if (lookDir == negY) {
      next_Y--;
    }

    if (next_X < 0 || next_X >= SIM_SIZE || next_Y < 0 || next_Y >= SIM_SIZE) { break; }

    check_X = next_X;
    check_Y = next_Y;
    distance++;
  }

  return distance;
}

