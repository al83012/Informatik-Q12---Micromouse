#ifndef drivecontrol_h
#define drivecontrol_h

namespace DRIVECONTROL {
    void forward(float distanceCm, float baseSpeedPercentage);
    void backward(float distanceCm, float baseSpeedPercentage);
    void setPIDParameters(float Kp, float Ki, float Kd);

}

#endif