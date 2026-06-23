#ifndef PID_H
#define PID_H

namespace pid {

    double calculate(double setpoint, double measured_value);
    void setTunings(double Kp, double Ki, double Kd);
    void setOutputLimits(double min, double max);
    void reset();
}


#endif
