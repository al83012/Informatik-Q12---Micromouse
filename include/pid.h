#ifndef PID_H
#define PID_H

class pid {
public:
    static double calculate(double setpoint, double measured_value);
    static void setTunings(double Kp, double Ki, double Kd);
    static void setOutputLimits(double min, double max);
    static void reset();
};


#endif
