#ifndef drivecontrol_h
#define drivecontrol_h

namespace DRIVECONTROL {
    constexpr float DEFAULT_BASE_SPEED_PERCENTAGE = 40.0f;
    constexpr float DEFAULT_TILE_SIZE_CM = 17.0f;

    constexpr float DEFAULT_TURN_SPEED_PERCENTAGE = 35.0f;

    void forward(float distanceCm, float baseSpeedPercentage);
    void backward(float distanceCm, float baseSpeedPercentage);
    void setPIDParameters(float Kp, float Ki, float Kd);
    void resetEncoders();

    void simpleForward1(float distanceCm, float baseSpeedPercentage);

    void defaultForward(int tiles);
    void defaultBackward(int tiles);

    void turn(float targetAngleDeg, float turnSpeedPercentage);

    void defaultTurnLeft(int turns, float angleDegrees);
    void defaultTurnRight(int turns, float angleDegrees);

    void correctAngle(float targetAngleDef, float correctionSpeedPercentage);
}

#endif