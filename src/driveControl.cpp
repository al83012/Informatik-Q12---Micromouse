#include "driveControl.h"
#include "Arduino.h"
#include "Components/drv8424.h"
#include "colors.h"
#include "Components/lsm6dsr.h"
#include "measurement.h"
#include "Components/iis2mdc.h"

using namespace COLORS;

namespace DRIVECONTROL {
     static float kp = 0.8;
     static float ki = 0.0;
     static float kd = 0.01;

     static float integral = 0.0;
     static float prevError = 0.0;
     static unsigned long prevTime = 0;

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
        if (dt <= 0.0) return 0.0;

        // P
        float pTerm = kp * error;

        // I
        integral += error * dt;
        float iTerm = ki * integral;

        // D
        float derivative = (error - prevError) / dt;
        float dTerm = kd * derivative;

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

    void forward(float distanceCm, float maxSpeedPercentage) {
        log_i("# (DRIVECONTROL) Moving forward: Distance =" CYAN "%f cm, Max Speed =" MAGENTA "%f%%", distanceCm, maxSpeedPercentage);
        long targetTicks = DRV8424::calculateTargetTicks(distanceCm);
        const float minSpeed = 8.0f;              
        const long accelTicks = targetTicks * 0.25; 
        const long decelTicks = targetTicks * 0.30;

        resetEncoders();
        resetPID();

        float currentHeadingErrorAccumulator = 0.0f;
        unsigned long lastLoopMicros = micros();
        const unsigned long loopIntervalMicros = 5000;
        unsigned long lastTofMillis = 0;
        int currentDist = 999;

        while (true) {
            
            unsigned long now = micros();
            if (now - lastLoopMicros < loopIntervalMicros) {
                continue;
            }
            float dt = (now - lastLoopMicros) / 1000000.0f;
            lastLoopMicros = now;
            unsigned long nowMillis = millis();

            // TOF-Check
            if (nowMillis - lastTofMillis >= 50) {
            lastTofMillis = nowMillis;
            currentDist = Measurement::TOF::secureMeasurementFront();
            }
            if (currentDist > 0 && currentDist < 100) {
            log_e(RED "# (DRIVECONTROL) Obstacle detected at %d mm! Stopping movement." RESET, currentDist);
            DRV8424::setSpeedPercentage1(0);
            DRV8424::setSpeedPercentage2(0);
            delay(15);
            break;
            }

         
            //Progress check
            long currentLeft = DRV8424::encoderCount1;
            long currentRight = DRV8424::encoderCount2;
            long averageProgress = (currentLeft + currentRight) / 2;
            
            if (averageProgress >= targetTicks) {
                break; 
            }
            
            // Acceleration at the beginning and deceleration at the end
            float currentBaseSpeed = maxSpeedPercentage;
            long ticksRemaining = targetTicks - averageProgress;
            if (averageProgress < accelTicks) { 
                currentBaseSpeed = minSpeed + (maxSpeedPercentage - minSpeed) * ((float)averageProgress / accelTicks);
            } else if (ticksRemaining < decelTicks) {
                currentBaseSpeed = minSpeed + (maxSpeedPercentage - minSpeed) * ((float)ticksRemaining / decelTicks);
            }

            LSM6DSR::Data sensorData = LSM6DSR::readSensorData();
            float correctedGyroZ = sensorData.gyroZ - LSM6DSR::ComponentVars::gyroBiasZ;

            //Gyro-Noise-Filter
            if (abs(correctedGyroZ) < 0.15f) {
            correctedGyroZ = 0.0f;
            }
            //Error computation using PID
            currentHeadingErrorAccumulator += correctedGyroZ * dt;
            float correction = computePID(currentHeadingErrorAccumulator);
            float speedLeft = currentBaseSpeed - correction;
            float speedRight = currentBaseSpeed + correction;    
            // Constrains for safety - might change later          
            speedLeft = constrain(speedLeft, -20.0f, 20.0f);
            speedRight = constrain(speedRight, -20.0f, 20.0f);   
            DRV8424::setSpeedPercentage1(-speedLeft); //Idk why but the motors are inverted so do not change
            DRV8424::setSpeedPercentage2(-speedRight);
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

void DRIVECONTROL::defaultForward(int tiles) {
    float distanceCm = tiles * DEFAULT_TILE_SIZE_CM;
    forward(distanceCm, DEFAULT_BASE_SPEED_PERCENTAGE);
}

void DRIVECONTROL::defaultBackward(int tiles) {
    float distanceCm = tiles * DEFAULT_TILE_SIZE_CM;
    backward(distanceCm, DEFAULT_BASE_SPEED_PERCENTAGE);
}

void DRIVECONTROL::turn(float angleDegrees, float maxSpeedPercentage) {
    log_i("# (DRIVECONTROL) Turning: Angle =" CYAN "%f deg, Max Speed =" MAGENTA "%f%%", angleDegrees, maxSpeedPercentage);
    
    const float minSpeed = 8.0f;
    float targetAngle = abs(angleDegrees);
    // turn direction
    float turnDirection = (angleDegrees >= 0.0f) ? 1.0f : -1.0f; 

    resetEncoders();
    resetPID();

    float accumulatedAngle = 0.0f;
    unsigned long lastLoopMicros = micros();
    const unsigned long loopIntervalMicros = 5000;

    while (true) {
        unsigned long now = micros();
        if (now - lastLoopMicros < loopIntervalMicros) {
            continue;
        }
        float dt = (now - lastLoopMicros) / 1000000.0f;
        lastLoopMicros = now;

        LSM6DSR::Data sensorData = LSM6DSR::readSensorData();
        float correctedGyroZ = sensorData.gyroZ - LSM6DSR::ComponentVars::gyroBiasZ;

        // Gyro-Noise filtering
        if (abs(correctedGyroZ) < 0.15f) {
            correctedGyroZ = 0.0f;
        }

        accumulatedAngle += abs(correctedGyroZ * dt);

        // Progress check
        if (accumulatedAngle >= targetAngle) {
            break; 
        }

        // Acceleration at the beginning and deceleration at the end based on angle progress
        float currentBaseSpeed = maxSpeedPercentage;
        float angleRemaining = targetAngle - accumulatedAngle;
        float accelAngle = targetAngle * 0.25f;
        float decelAngle = targetAngle * 0.30f;

        if (accumulatedAngle < accelAngle) { 
            currentBaseSpeed = minSpeed + (maxSpeedPercentage - minSpeed) * (accumulatedAngle / accelAngle);
        } else if (angleRemaining < decelAngle) {
            currentBaseSpeed = minSpeed + (maxSpeedPercentage - minSpeed) * (angleRemaining / decelAngle);
        }

        // Counter rotation
        float speedLeft = currentBaseSpeed * turnDirection;
        float speedRight = -currentBaseSpeed * turnDirection;

           
        speedLeft = constrain(speedLeft, -20.0f, 20.0f);
        speedRight = constrain(speedRight, -20.0f, 20.0f);   

        DRV8424::setSpeedPercentage1(-speedLeft); 
        DRV8424::setSpeedPercentage2(-speedRight);
    }

    DRV8424::setSpeedPercentage1(0);
    DRV8424::setSpeedPercentage2(0);
}