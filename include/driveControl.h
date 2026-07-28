#ifndef drivecontrol_h
#define drivecontrol_h

namespace DRIVECONTROL {
    constexpr float DEFAULT_BASE_SPEED_PERCENTAGE = 15.0f;
    constexpr float DEFAULT_TILE_SIZE_CM = 17.0f;

    constexpr float DEFAULT_TURN_SPEED_PERCENTAGE = 7.0f;

    void forward(float distanceCm, float baseSpeedPercentage);
    void backward(float distanceCm, float baseSpeedPercentage);
    void setPIDParameters(float Kp, float Ki, float Kd);
    void resetEncoders();

    void simpleForward1(float distanceCm, float baseSpeedPercentage);

    void defaultForward(int tiles);
    void defaultBackward(int tiles);

    void turnLeft(float angleDegrees, float baseSpeedPercentage);
    void turnRight(float angleDegrees, float baseSpeedPercentage);

    void defaultTurnLeft(int turns, float angleDegrees);
    void defaultTurnRight(int turns, float angleDegrees);
}

#endif