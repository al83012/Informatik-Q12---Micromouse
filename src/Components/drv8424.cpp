
#include "components/drv8424.h"
#include "components/tpl0102.h"
#include "Arduino.h"
#include "Wire.h"
#include "Components/tcal6408.h"
#include "colors.h"
using namespace COLORS;

namespace DRV8424 {
    volatile long encoderCount1 = 0;
    volatile long encoderCount2 = 0;
    volatile bool direction1 = true;
    volatile bool direction2 = true;

    uint8_t dutyCycle1 = 0;
    uint8_t dutyCycle2 = 0;
    int frequency = 50000;
}

    
void DRV8424::init(int frequency) {
    log_d("# (DRV8424) Initializing DRV8424 with frequency: %d", frequency);
    pinMode(ENC_A1, INPUT);
    pinMode(ENC_A2, INPUT); 

    pinMode(ENC_B1, INPUT);
    pinMode(ENC_B2, INPUT); 

    pinMode(DRV_AIN1, OUTPUT);
    pinMode(DRV_AIN2, OUTPUT);
    pinMode(DRV_BIN1, OUTPUT);
    pinMode(DRV_BIN2, OUTPUT);

    TCAL6408::setDriverAwake(true);

    attachInterrupt(digitalPinToInterrupt(ENC_A1), DRV8424::readEncoder1, RISING);
    attachInterrupt(digitalPinToInterrupt(ENC_A2), DRV8424::readEncoder2, RISING);

    setFrequency(frequency);
  
}

bool DRV8424::assureSafeCurrent() {
    log_d("# (DRV8424) Assuring safe current...");
    return TPL0102::setCurrentLimitDriver(maxCurrent);
}

void DRV8424::readEncoder1() {
    bool encB1 = digitalRead(ENC_B1);

    if(encB1 == HIGH) {
        encoderCount1++;
        direction1 = true;
    } else {
        encoderCount1--;
        direction1 = false;
    }
    
   // log_d("Interrupt: ENC_A1 changed. Encoder Count 1: %ld, Direction: %s", encoderCount1, direction1 ? "Forward" : "Reverse");
}

void DRV8424::readEncoder2() {
    bool encB2 = digitalRead(ENC_B2);

    if(encB2 == HIGH) {
        encoderCount2++;
        direction2 = true;
    } else {
        encoderCount2--;
        direction2 = false;
    }

 //   log_d("Interrupt: ENC_A2 changed. Encoder Count 2: %ld, Direction: %s", encoderCount2, direction2 ? "Forward" : "Reverse");

}

void DRV8424::debugPrintEncoderCounts() {
    log_i("---------DRV8424 ENCODER COUNTS (DRV8424)---------");
    log_i("# ENC_1:" CYAN "%ld," RESET "Direction: " MAGENTA "%s" RESET, encoderCount1, direction1 ? "Forward" : "Reverse" );
    log_i("# ENC_2: " CYAN "%ld," RESET "Direction: " MAGENTA "%s" RESET, encoderCount2, direction2 ? "Forward" : "Reverse" );
    log_i("---------------------------------------");
}

void DRV8424::setDutyCycle1(int dutyCycle) {
    log_d("# (DRV8424) Setting duty cycle 1 to %d", dutyCycle);
    dutyCycle = constrain(dutyCycle, -255, 255);
    
    if (dutyCycle >= 0) {
        analogWrite(DRV_AIN1, dutyCycle);
        analogWrite(DRV_AIN2, 0);
        dutyCycle1 = dutyCycle;
    } else {
        analogWrite(DRV_AIN1, 0);
        analogWrite(DRV_AIN2, abs(dutyCycle));
        dutyCycle1 = abs(dutyCycle);
    }
}

void DRV8424::setDutyCycle2(int dutyCycle) {
    log_d("# (DRV8424) Setting duty cycle 2 to %d", dutyCycle);
    dutyCycle = constrain(dutyCycle, -255, 255);
    
    if (dutyCycle >= 0) {
        analogWrite(DRV_BIN1, dutyCycle);
        analogWrite(DRV_BIN2, 0);
        dutyCycle2 = dutyCycle;
    } else {
        analogWrite(DRV_BIN1, 0);
        analogWrite(DRV_BIN2, abs(dutyCycle));
        dutyCycle2 = abs(dutyCycle);
    }
}

void DRV8424::setFrequency(int frequency) {
    log_d("# (DRV8424) Setting frequency to %d", frequency);
    DRV8424::frequency = frequency;
    analogWriteFrequency(frequency);
}



void DRV8424::driveDistance(float distanceCm, float speedPercentage) {
    log_i("# (DRV8424) Driving distance: %f cm at speed percentage: %f", distanceCm, speedPercentage);
    if(!DRV8424::assureSafeCurrent()) {
        log_e("# Current limit not safe!");
        return;
    }

    if(distanceCm == 0) {
        log_i("# (DRV8424) Distance is zero, no movement required.");
        return;
    }

    if(speedPercentage == 0) {
        log_i("# (DRV8424) Speed percentage is zero, no movement required.");
        return;
    }

    long requiredTicks = calculateTargetTicks(distanceCm);
    
    long target_tick1 = encoderCount1 + requiredTicks;    
    long target_tick2 = encoderCount2 + requiredTicks;

    bool motor1_active = (requiredTicks != 0);
    bool motor2_active = (requiredTicks != 0);

    if (requiredTicks > 0) {
        setSpeedPercentage1(abs(speedPercentage));  // Motor 1 vorwärts
        setSpeedPercentage2(abs(speedPercentage));  // Motor 2 vorwärts
    } else {
        setSpeedPercentage1(-abs(speedPercentage)); // Motor 1 rückwärts
        setSpeedPercentage2(-abs(speedPercentage)); // Motor 2 rückwärts
    }

    while (motor1_active || motor2_active) {

        
        
        //Motor 1
        if (motor1_active) {
            if ((requiredTicks > 0 && encoderCount1 >= target_tick1) || 
                (requiredTicks < 0 && encoderCount1 <= target_tick1)) {
                setDutyCycle1(0); // Motor 1 stoppen
                motor1_active = false;
            }
        }

        //Motor 2
        if (motor2_active) {
            if ((requiredTicks > 0 && encoderCount2 >= target_tick2) || 
                (requiredTicks < 0 && encoderCount2 <= target_tick2)) {
                setDutyCycle2(0); // Motor 2 stoppen
                motor2_active = false;
            }
        }

        delay(1); 
    }
}

void DRV8424::setSpeedPercentage1(float speedPercentage) {
    log_d("# (DRV8424) Setting speed percentage 1 to %f", speedPercentage);
    speedPercentage = constrain(speedPercentage, -100.0f, 100.0f);
    int dutyCycle = static_cast<int>((speedPercentage / 100.0f) * 255);
    setDutyCycle1(dutyCycle);
}

void DRV8424::setSpeedPercentage2(float speedPercentage) {
    log_d("# (DRV8424) Setting speed percentage 2 to %f", speedPercentage);
    speedPercentage = constrain(speedPercentage, -100.0f, 100.0f);
    int dutyCycle = static_cast<int>((speedPercentage / 100.0f) * 255);
    setDutyCycle2(dutyCycle);
}


long DRV8424::calculateTargetTicks(float distanceCm) {
    
    log_d("# (DRV8424) Calculating target ticks for distance: %f cm", distanceCm);
    float totalTicks = (distanceCm / DRV8424::wheelCircumference) * DRV8424::stepsPerRevolution;
    
    return (long)(totalTicks + 0.5); 
}
