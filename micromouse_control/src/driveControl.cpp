#include "driveControl.h"
#include "Arduino.h"
#include "Components/drv8424.h"
#include "colors.h"

using namespace COLORS;

namespace DRIVECONTROL {
     float kp = 0.6;
     float ki = 0.0;
     float kd = 0.1;

     float integral = 0.0;
     float prevError = 0.0;
     unsigned long prevTime = 0;

     static void resetPID() {
        integral = 0.0;
        prevError = 0.0;
        prevTime = millis();
        log_d("# (DRIVECONTROL) PID reset: integral=%f, prevError=%f, prevTime=%lu", integral, prevError, prevTime);
    }

    static float computePID(float error) {

        unsigned long currentTime = millis();
        float dt = (currentTime - prevTime) / 1000.0; // Zeit in Sekunden
        dt = constrain(dt, 0.001, 1.0);
        log_i(GREEN "Constrained dt: %f seconds" RESET, dt);
        if (dt <= 0.0) return 0.0; // UNREACHABLE: constrain(dt, 0.001, 1.0); (Außerdem ist der Output 0.0 nicht passend)

        // P
        float pTerm = kp * error;

        // I
        integral += error * dt;
        float iTerm = ki * integral;

        // D
        float derivative = (error - prevError) / dt;
        float dTerm = kd * derivative; // Vllt: Constrain auf dem dTerm; Sonst kann eine schnelle Anpassung des target- und damit des error- werts zu plötzlichen Spitzen führen

        prevError = error;
        prevTime = currentTime;
        log_i("# (DRIVECONTROL) PID Computation: P=%f, I=%f, D=%f, Output=%f", pTerm, iTerm, dTerm, pTerm + iTerm + dTerm);
        return pTerm + iTerm + dTerm;
    }

    void setPIDParameters(float kP, float kI, float kD) {
        log_d("# (DRIVECONTROL) Setting PID parameters: Kp=%f, Ki=%f, Kd=%f", kP, kI, kD);
        kp = kP;
        ki = kI;
        kd = kD;
    }


    void resetEncoders() {
        log_d("# (DRIVECONTROL) Resetting encoders...");
        noInterrupts(); 
        DRV8424::encoderCount1 = 0;
        DRV8424::encoderCount2 = 0;
        interrupts();   
        log_d("# (DRIVECONTROL) Encoders reset: encoderCount1=%ld, encoderCount2=%ld", DRV8424::encoderCount1, DRV8424::encoderCount2);
    }

    void simpleForward1(float distanceCm, float baseSpeedPercentage) {
        log_i("# (DRIVECONTROL) Moving (simple1) forward: Distance =" CYAN "%f cm, Base Speed =" MAGENTA "%f%%", distanceCm, baseSpeedPercentage);
        long targetTicks = DRV8424::calculateTargetTicks(distanceCm);
        log_i("# Target-ticks: %d", targetTicks);
        resetEncoders();
        while(true) {
            long currentLeft = DRV8424::encoderCount1;
            long currentRight = DRV8424::encoderCount2;

            if(abs(currentLeft) >= targetTicks && abs(currentRight) >= targetTicks) {
                log_i(GREEN "# Movement finished succesfuly!" RESET);
                DRV8424::setSpeedPercentage1(0);
                DRV8424::setSpeedPercentage2(0);
                break;
            }
            DRV8424::setSpeedPercentage1(baseSpeedPercentage);
            DRV8424::setSpeedPercentage2(baseSpeedPercentage);
            

        }

    }    

    void forward(float distanceCm, float baseSpeedPercentage) {
        log_i("# (DRIVECONTROL) Moving forward: Distance =" CYAN "%f cm, Base Speed =" MAGENTA "%f%%", distanceCm, baseSpeedPercentage);
        long targetTicks = DRV8424::calculateTargetTicks(distanceCm);
        
        resetEncoders();
        resetPID();


        while (true) {
            long currentLeft = DRV8424::encoderCount1;
            long currentRight = DRV8424::encoderCount2;
            
            long averageProgress = (currentLeft + currentRight) / 2;
            if (averageProgress >= targetTicks) {
                break; 
            }


            float error = (float)(currentLeft - currentRight);

            float correction = computePID(error);
            
            DRV8424::debugPrintEncoderCounts();
           
            float speedLeft = baseSpeedPercentage - correction;     
            float speedRight = baseSpeedPercentage + correction;    

            speedLeft = constrain(speedLeft, 0, 10);   
            speedRight = constrain(speedRight, 0, 10); 

            DRV8424::setSpeedPercentage1(speedLeft);    
            DRV8424::setSpeedPercentage2(speedRight);   

            delay(10); 
        }

        DRV8424::setSpeedPercentage1(0);
        DRV8424::setSpeedPercentage2(0);
}


void backward(float distanceCm, float baseSpeedPercentage) {
    log_i("# (DRIVECONTROL) Moving backward: Distance =" CYAN "%f cm, Base Speed =" MAGENTA "%f%%", distanceCm, baseSpeedPercentage);
    long targetTicks = DRV8424::calculateTargetTicks(distanceCm);
    
    DRV8424::encoderCount1 = 0;
    DRV8424::encoderCount2 = 0;
    resetPID();

    while (true) {
        long currentLeft = DRV8424::encoderCount1;
        long currentRight = DRV8424::encoderCount2;
        
    
        long averageProgress = (currentLeft + currentRight) / 2;
        
        if (averageProgress <= -targetTicks) {
            break; 
        }

  
        float error = (float)(currentLeft - currentRight);

        float correction = computePID(error);

      
        float speedLeft = -baseSpeedPercentage - correction;
        float speedRight = -baseSpeedPercentage + correction;

        speedLeft = constrain(speedLeft, -100, 0);
        speedRight = constrain(speedRight, -100, 0);

        DRV8424::setSpeedPercentage1(speedLeft);
        DRV8424::setSpeedPercentage2(speedRight);

        delay(10); 
    }

    DRV8424::setSpeedPercentage1(0);
    DRV8424::setSpeedPercentage2(0);
}

}   
