#ifndef MOVEMENT_H
#define MOVEMENT_H
#include "Arduino.h"

namespace Movement {

namespace Maze {
    constexpr double baseSpeed = 1;
    constexpr double squareLength = 16.8; // in cm
    constexpr double maxSpeed = 1; // in m/s
    constexpr double emergencyBreakDistance = 3; // in cm
}

namespace Status {
    bool centered = false;
    mode Mode = mode::simpleMove;
}

namespace Motor {
    double speedLeft;
    double speedRight;
}

enum mode {
    simpleMove,
    simpleMoveCentered,
    simplePid,
    simplePidCentered,
    advancedPid,
    advancedPidCentered
};
}

namespace Cell {
    double X;
    double Y;
    double distanceL;
    double distanceR;

}

void moveForward(int distance, bool safetyMode);
void moveBackward(int distance, bool safetyMode);

void setMovementMode(Movement::mode mode);
void turnLeft();
void turnRight();



#endif