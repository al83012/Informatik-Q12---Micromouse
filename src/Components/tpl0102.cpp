#include "Components/TPL0102.h"
#include "Arduino.h"
#include "Components/esp32.h"
#include "i2ctool.h"

using namespace I2CTOOL;
using namespace TPL0102;
using namespace Measurement::Sensors;

// Potentiometer

void TPL0102::init(float highVoltage) {
    ComponentVars::highVoltage = highVoltage;
    findComponent(SensorNames::TPL0102_POTENTIOMETER);

    int ex, en, di;

    ex = exitShutdown();
    en = enableNonVolatileWriting();

    ComponentVars::DefaultWiperPosA =  getWiperA();
    ComponentVars::wiperPosA =  getWiperA();
    ComponentVars::DefaultWiperPosB =  getWiperB();
    ComponentVars::wiperPosB =  getWiperB();

    di = disableNonVolatileWriting();
    
    if(ex == -1 || en == -1 || di == -1 ||ComponentVars::DefaultWiperPosA  == -1 || ComponentVars::wiperPosA == -1 || ComponentVars::DefaultWiperPosB == -1 || ComponentVars::wiperPosB == -1){
        e_sensor(SensorNames::TPL0102_POTENTIOMETER, "Failed to configure!");
    }
    else{
        i_sensor(SensorNames::TPL0102_POTENTIOMETER, "Configured succesfully");

    }

}

int TPL0102::SetVolatileWiperA(uint8_t position) {
     log_d("# Trying to set WiperA to %d...", position);

    if(canWriteAutoRetry) {
        log_d("Setting WiperA...");
        I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_WIP_A, position);
    } else {
        e_sensor(SensorNames::TPL0102_POTENTIOMETER, "Failed to set WiperA!");
        return -1;
    }

    ComponentVars::wiperPosA = position;
    log_d("# WiperA set succesfully!");
    return 0;
}

int TPL0102::SetVolatileWiperB(uint8_t position) {
     log_d("# Trying to set WiperB to %d...", position);
    if(canWriteAutoRetry) {
        log_d("Setting WiperB...");
        I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_WIP_B, position);
    } else {
        e_sensor(SensorNames::TPL0102_POTENTIOMETER, "Failed to set WiperA!");
        return -1;
    }
    
    ComponentVars::wiperPosB = position;
    log_d("# WiperB set succesfully!");
    return 0;
}


int TPL0102::SetNonVolatileWiperA(uint8_t position){
    log_d("# Trying to set non-volatile WiperA to %d...", position);

    if(enableNonVolatileWriting() == -1){
        e_sensor(SensorNames::TPL0102_POTENTIOMETER, "Failed to set non-volatile WiperA!");

        return -1;
    }

    if(canWriteAutoRetry()){
        log_d("# Setting non-volatile WiperA...");
        I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_WIP_A, position);
    }
    else{
        log_e("Failed to set non-volatile WiperA!");
        return -1;
    }

    ComponentVars::DefaultWiperPosA = position;

    if(disableNonVolatileWriting() == -1){
        log_e("# Set non-volatile WiperA, but failed to disable non-volatile Writing!");
        return -1;
    }

    log_d("# Non-volatile WiperA set successfully.");
    return 0;
}


int TPL0102::SetNonVolatileWiperB(uint8_t position){
    log_d("# Trying to set non-volatile WiperB to %d...", position);

    if(enableNonVolatileWriting() == -1){
        log_e("# Failed to set non-volatile WiperB!");
        return -1;
    }

    if(canWriteAutoRetry()){
        log_d("# Setting non-volatile WiperB...");
        I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_WIP_B, position);
    }
    else{
        log_e("Failed to set non-volatile WiperB!");
        return -1;
    }

    ComponentVars::DefaultWiperPosB = position;

    if(disableNonVolatileWriting() == -1){
        log_e("# Set non-volatile WiperB, but failed to disable non-volatile Writing!");
        return -1;
    }

    log_d("# Non-volatile WiperB set successfully.");
    return 0;
}

int TPL0102::getWiperA(){
    log_d("# Reading WiperA...");

    ComponentVars::wiperPosA;
    I2C1Read(ComponentVars::I2C_ADDRESS, ComponentVars::REG_WIP_A, ComponentVars::wiperPosA);

    log_d("WiperA read successfully: %d", ComponentVars::wiperPosA);

    return ComponentVars::wiperPosA;
}


int TPL0102::getWiperB(){
    log_d("# Reading WiperB...");

    ComponentVars::wiperPosB;
     I2C1Read(ComponentVars::I2C_ADDRESS, ComponentVars::REG_WIP_B, ComponentVars::wiperPosB);

    log_d("# WiperB read successfully: %d", ComponentVars::wiperPosB);

    return ComponentVars::wiperPosB;
}


int TPL0102::enableNonVolatileWriting(){
    log_d("# Trying to enable non-volatile writing...");

    if(canWriteAutoRetry()){
        uint8_t output = 0b01000000;
        if(ComponentVars::shutdownEnabled){
            output = 0b00000000;
        }

        log_d("# Enabling non-volatile writing...");
        I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_SETTINGS, output);
        

    }
    else{
        log_e("# Failed to enable non-volatile writing!");
        return -1;
    }

    log_d("# Non-volatile writing enabled successfully.");
    return 0;
}

int TPL0102::disableNonVolatileWriting(){
    log_d("# Trying to disable non-volatile writing...");

    if(canWriteAutoRetry()){
        uint8_t output = 0b11000000;
        if(ComponentVars::shutdownEnabled){
            output = 0b10000000;
        }

        log_d("# Enabling non-volatile writing...");
        I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_SETTINGS, output);
        

    }
    else{
        log_e("# Failed to disable non-volatile writing!");
        return -1;
    }

    log_d("# Non-volatile writing disabled successfully.");
    return 0;
}


int TPL0102::canWrite(){
    log_d("# Checking if writable...");


    uint8_t currentSettings;
    I2C1Read(ComponentVars::I2C_ADDRESS, ComponentVars::REG_SETTINGS, currentSettings);

    uint8_t WIP = (currentSettings & ComponentVars::SETTING_WIPMask) >> 5;


    log_d("# WIP read successfully: %d", WIP);

    if(WIP){
        return 0;
    }
    return 1;
}


int TPL0102::canWriteAutoRetry(){
    log_d("# Autor-Retry Checking if writable...");

    for(int i = 0; i < ComponentVars::canWriteAutoRetryAttempts; i++){
        int canW = canWrite();
        if(canW && canW != -1){
            return 1;
        }
        else if(canW == -1){
            log_e("# Something went wrong, waiting %dms and trying again!", ComponentVars::canWriteAutoRetryAttempts);
            delay(ComponentVars::canWriteAutoRetryAttempts);
        }
        else{
            log_e("# WIP is set high, waiting %dms and trying again!", ComponentVars::canWriteAutoRetryDelay);
            delay(ComponentVars::canWriteAutoRetryDelay);
        }
    }

    log_e("# After %d attempts there was no canWrite resolution!", ComponentVars::canWriteAutoRetryAttempts);
    return 0;
}


int TPL0102::setVoltageA(float voltage){
    log_d("# Trying to set VoltageA to %fV", voltage);

    if(voltage > ComponentVars::highVoltage){
        log_e("Voltage target is higher that highVoltage!");
        log_e("Failed to set VoltageA!");
        return -1;
    }

    if(voltage < 0.0f){
        log_e("# Voltage target is negative!");
        log_e("# Failed to set VoltageA!");
        return -1;
    }

    float estPosf = (voltage / ComponentVars::highVoltage) * 256;
    estPosf = std::roundf(estPosf);
    if(estPosf == 256) estPosf--;

    uint8_t estPos = static_cast<uint8_t>(estPosf);


    if(SetVolatileWiperA(estPos) == -1){
        log_e("# Failed to set VoltageA!");
        return -1;
    }

    float actualVoltage = ComponentVars::highVoltage * (estPosf / 256);
    log_d("# VoltageA set to %fV", actualVoltage);
    return 0;
}

int TPL0102::setVoltageB(float voltage){
    log_d("# Trying to set VoltageB to %fV", voltage);

    if(voltage > ComponentVars::highVoltage){
        log_e("# Voltage target is higher that highVoltage!");
        log_e("# Failed to set VoltageB!");
        return -1;
    }

    if(voltage < 0.0f){
        log_e("# Voltage target is negative!");
        log_e("# Failed to set VoltageB!");
        return -1;
    }

    float estPosf = (voltage / ComponentVars::highVoltage) * 256;
    estPosf = std::round(estPosf);
    if(estPosf == 256) estPosf--;

    uint8_t estPos = static_cast<uint8_t>(estPosf);

    if(SetVolatileWiperB(estPos) == -1){
        log_e("# Failed to set VoltageB!");
        return -1;
    }

    float actualVoltage = ComponentVars::highVoltage * (estPosf / 256);
    log_d("VoltageB set to %fV", actualVoltage);
    return 0;
}

int TPL0102::setDefaultVoltageA(float voltage){
    log_d("# Trying to set Default-VoltageA to %fV", voltage);

    if(voltage > ComponentVars::highVoltage){
        log_e("# Voltage target is higher that highVoltage!");
        log_e("# Failed to set Default-VoltageA!");
        return -1;
    }

    if(voltage < 0.0f){
        log_e("# Voltage target is negative!");
        log_e("# Failed to set Default-VoltageA!");
        return -1;
    }

    float estPosf = (voltage / ComponentVars::highVoltage) * 256;
    estPosf = std::round(estPosf);
    if(estPosf == 256) estPosf--;

    uint8_t estPos = static_cast<uint8_t>(estPosf);

    if(SetNonVolatileWiperA(estPos) == -1){
        log_e("# Failed to set Default-VoltageA!");
        return -1;
    }

    float actualVoltage = ComponentVars::highVoltage * (estPosf / 256);
    log_d("Default-VoltageA set to %fV", actualVoltage);
    return 0;
}

int TPL0102::setDefaultVoltageB(float voltage){
    log_d("# Trying to set Default-VoltageB to %fV", voltage);
    
    if(voltage > ComponentVars::highVoltage){
        log_e("# Voltage target is higher that highVoltage!");
        log_e("# Failed to set Default-VoltageB!");
        return -1;
    }

    if(voltage < 0.0f){
        log_e("# Voltage target is negative!");
        log_e("# Failed to set Default-VoltageB!");
        return -1;
    }

    float estPosf = (voltage / ComponentVars::highVoltage) * 256;
    estPosf = std::round(estPosf);
    if(estPosf == 256) estPosf--;

    uint8_t estPos = static_cast<uint8_t>(estPosf);

    if(SetNonVolatileWiperB(estPos) == -1){
        log_e("# Failed to set Default-VoltageB!");
        return -1;
    }

    float actualVoltage = ComponentVars::highVoltage * (estPosf / 256);
    log_d("# Default-VoltageB set to %fV", actualVoltage);
    return 0;
}


float TPL0102::getVoltageA(){
    float posf = static_cast<float>(ComponentVars::wiperPosA);

    float voltage = ComponentVars::highVoltage * (posf / 256);

    log_d("VoltageA is %fV", voltage);
    return voltage;
}

float TPL0102::getVoltageB(){
    float posf = static_cast<float>(ComponentVars::wiperPosB);

    float voltage = ComponentVars::highVoltage * (posf / 256);

    log_d("VoltageB is %fV", voltage);
    return voltage;
}

float TPL0102::getDefaultVoltageA(){
    float posf = static_cast<float>(ComponentVars::DefaultWiperPosA);

    float voltage = ComponentVars::highVoltage * (posf / 256);

    log_d("Default-VoltageA is %fV", voltage);
    return voltage;
}

float TPL0102::getDefaultVoltageB(){
    float posf = static_cast<float>(ComponentVars::DefaultWiperPosB);

    float voltage = ComponentVars::highVoltage * (posf / 256);

    log_d("Default-VoltageB is %fV", voltage);
    return voltage;
}


int TPL0102::enterShutdown(){
    log_d("# Trying to enter shutdown");
    
    if(canWriteAutoRetry()){
        uint8_t output = 0b10000000;

        log_d("# Entering Shutdown...");
        I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_SETTINGS, output);
    }
    else{
        log_e("# Failed to enter shutdown!");
        return -1;
    }

    ComponentVars::shutdownEnabled = true;

    log_d("# Entered shutdown successfully.");
    return 0;
}


int TPL0102::exitShutdown(){
    log_d("# Trying to exit shutdown");
    
    if(canWriteAutoRetry()){
        uint8_t output = 0b11000000;

        log_d("Exiting Shutdown...");
        I2C1Write(ComponentVars::I2C_ADDRESS, ComponentVars::REG_SETTINGS, output);
    }
    else{
        log_e("Failed to exit shutdown!");
        return -1;
    }

    ComponentVars::shutdownEnabled = false;

    log_d("Exited shutdown successfully.");
    return 0;
}

float TPL0102::getHighVoltage(){
    return ComponentVars::highVoltage;
}

void TPL0102::DbgPrintVoltages() {
 float voltage = getVoltageA();
 
  log_i("VoltageA: %f", voltage);
  setVoltageA(3.0f);
  voltage = getVoltageA();
  log_i("VoltageA: %f", voltage);


  voltage = getVoltageB();
  log_i("VoltageB: %f", voltage);

  setVoltageB(3.3f);
  voltage = getVoltageB();
  log_i("VoltageB: %f", voltage);

  enterShutdown();
  delay(10000);
  exitShutdown();
}