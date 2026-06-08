#include "Components/TPL0102.h"
#include "Arduino.h"
#include "Components/esp32.h"
#include "i2ctool.h"

// Potentiometer
TPL0102_ComponentVars tpl0102_componentVars;

void TPL0102::init(float highVoltage) {
     Wire.beginTransmission(tpl0102_componentVars.I2C_ADDRESS);
    if (Wire.endTransmission() != 0) {
        log_e("# TPL0102 (Potentiometer) not found!");
    } else {
        log_i("# TPL0102 (Potentiometer) initialized successfully");
    }

    int ex, en, di;

    ex = exitShutdown();
    en = enableNonVolatileWriting();

    tpl0102_componentVars.DefaultWiperPosA, tpl0102_componentVars.wiperPosA =  getWiperA();
    tpl0102_componentVars.DefaultWiperPosB, tpl0102_componentVars.wiperPosB =  getWiperB();

    di = disableNonVolatileWriting();
    
    if(ex == -1 || en == -1 || di == -1 ||tpl0102_componentVars.DefaultWiperPosA  == -1 || tpl0102_componentVars.wiperPosA == -1 || tpl0102_componentVars.DefaultWiperPosB == -1 || tpl0102_componentVars.wiperPosB == -1){
        log_e("# Failed to configure TPL0102!");
    }
    else{
        log_i("# Configured TPL0102 succesfully!");
    }

}

int TPL0102::SetVolatileWiperA(uint8_t position) {
     log_d("# Trying to set WiperA to %d...", position);
    if(canWriteAutoRetry) {
        log_d("Setting WiperA...");
        I2CTOOL::i2c_writeRegister(tpl0102_componentVars.I2C_ADDRESS, tpl0102_componentVars.REG_WIP_A, position);
    } else {
        log_e("# Failed to set WiperA!");
        return -1;
    }

    tpl0102_componentVars.wiperPosA = position;
    log_d("# WiperA set succesfully!");
}

int TPL0102::SetVolatileWiperB(uint8_t position) {
     log_d("# Trying to set WiperB to %d...", position);
    if(canWriteAutoRetry) {
        log_d("Setting WiperB...");
        I2CTOOL::i2c_writeRegister(tpl0102_componentVars.I2C_ADDRESS, tpl0102_componentVars.REG_WIP_B, position);
    } else {
        log_e("# Failed to set WiperA!");
        return -1;
    }

    tpl0102_componentVars.wiperPosB = position;
    log_d("# WiperB set succesfully!");
}


int TPL0102::SetNonVolatileWiperA(uint8_t position){
    log_d("# Trying to set non-volatile WiperA to %d...", position);

    if(enableNonVolatileWriting() == -1){
        log_e("# Failed to set non-volatile WiperA!");
        return -1;
    }

    if(canWriteAutoRetry()){
        log_d("# Setting non-volatile WiperA...");
        I2CTOOL::i2c_writeRegister(tpl0102_componentVars.I2C_ADDRESS, tpl0102_componentVars.REG_WIP_A, position);
    }
    else{
        log_e("Failed to set non-volatile WiperA!");
        return -1;
    }

    tpl0102_componentVars.DefaultWiperPosA = position;

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
        I2CTOOL::i2c_writeRegister(tpl0102_componentVars.I2C_ADDRESS, tpl0102_componentVars.REG_WIP_B, position);
    }
    else{
        log_e("Failed to set non-volatile WiperB!");
        return -1;
    }

    tpl0102_componentVars.DefaultWiperPosB = position;

    if(disableNonVolatileWriting() == -1){
        log_e("# Set non-volatile WiperB, but failed to disable non-volatile Writing!");
        return -1;
    }

    log_d("# Non-volatile WiperB set successfully.");
    return 0;
}

int TPL0102::getWiperA(){
    log_d("# Reading WiperA...");

    tpl0102_componentVars.wiperPosA = I2CTOOL::i2c_readRegister(tpl0102_componentVars.I2C_ADDRESS, tpl0102_componentVars.REG_WIP_A);

    log_d("WiperA read successfully: %d", tpl0102_componentVars.wiperPosA);

    return tpl0102_componentVars.wiperPosA;
}


int TPL0102::getWiperB(){
    log_d("# Reading WiperB...");

    tpl0102_componentVars.wiperPosB = I2CTOOL::i2c_readRegister(tpl0102_componentVars.I2C_ADDRESS, tpl0102_componentVars.REG_WIP_B);

    log_d("# WiperB read successfully: %d", tpl0102_componentVars.wiperPosB);

    return tpl0102_componentVars.wiperPosB;
}


int TPL0102::enableNonVolatileWriting(){
    log_d("# Trying to enable non-volatile writing...");

    if(canWriteAutoRetry()){
        uint8_t output = 0b01000000;
        if(tpl0102_componentVars.shutdownEnabled){
            output = 0b00000000;
        }

        log_d("# Enabling non-volatile writing...");
        I2CTOOL::i2c_writeRegister(tpl0102_componentVars.I2C_ADDRESS, tpl0102_componentVars.REG_SETTINGS, output);
        

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
        if(tpl0102_componentVars.shutdownEnabled){
            output = 0b10000000;
        }

        log_d("# Enabling non-volatile writing...");
        I2CTOOL::i2c_writeRegister(tpl0102_componentVars.I2C_ADDRESS, tpl0102_componentVars.REG_SETTINGS, output);
        

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


    uint8_t currentSettings = I2CTOOL::i2c_readRegister(tpl0102_componentVars.I2C_ADDRESS, tpl0102_componentVars.REG_SETTINGS);

    uint8_t WIP = (currentSettings & tpl0102_componentVars.SETTING_WIPMask) >> 5;


    log_d("# WIP read successfully: %d", WIP);

    if(WIP){
        return 0;
    }
    return 1;
}


int TPL0102::canWriteAutoRetry(){
    log_d("# Autor-Retry Checking if writable...");

    for(int i = 0; i < tpl0102_componentVars.canWriteAutoRetryAttempts; i++){
        int canW = canWrite();
        if(canW && canW != -1){
            return 1;
        }
        else if(canW == -1){
            log_e("# Something went wrong, waiting %dms and trying again!", tpl0102_componentVars.canWriteAutoRetryAttempts);
            delay(tpl0102_componentVars.canWriteAutoRetryAttempts);
        }
        else{
            log_e("# WIP is set high, waiting %dms and trying again!", tpl0102_componentVars.canWriteAutoRetryDelay);
            delay(tpl0102_componentVars.canWriteAutoRetryDelay);
        }
    }

    log_e("# After %d attempts there was no canWrite resolution!", tpl0102_componentVars.canWriteAutoRetryAttempts);
    return 0;
}


int TPL0102::setVoltageA(float voltage){
    log_d("# Trying to set VoltageA to %fV", voltage);

    if(voltage > tpl0102_componentVars.highVoltage){
        log_e("Voltage target is higher that highVoltage!");
        log_e("Failed to set VoltageA!");
        return -1;
    }

    if(voltage < 0.0f){
        log_e("# Voltage target is negative!");
        log_e("# Failed to set VoltageA!");
        return -1;
    }

    float estPosf = (voltage / tpl0102_componentVars.highVoltage) * 256;
    estPosf = std::roundf(estPosf);
    if(estPosf == 256) estPosf--;

    uint8_t estPos = static_cast<uint8_t>(estPosf);


    if(SetVolatileWiperA(estPos) == -1){
        log_e("# Failed to set VoltageA!");
        return -1;
    }

    float actualVoltage = tpl0102_componentVars.highVoltage * (estPosf / 256);
    log_d("# VoltageA set to %fV", actualVoltage);
    return 0;
}

int TPL0102::setVoltageB(float voltage){
    log_d("# Trying to set VoltageB to %fV", voltage);

    if(voltage > tpl0102_componentVars.highVoltage){
        log_e("# Voltage target is higher that highVoltage!");
        log_e("# Failed to set VoltageB!");
        return -1;
    }

    if(voltage < 0.0f){
        log_e("# Voltage target is negative!");
        log_e("# Failed to set VoltageB!");
        return -1;
    }

    float estPosf = (voltage / tpl0102_componentVars.highVoltage) * 256;
    estPosf = std::round(estPosf);
    if(estPosf == 256) estPosf--;

    uint8_t estPos = static_cast<uint8_t>(estPosf);

    if(SetVolatileWiperB(estPos) == -1){
        log_e("# Failed to set VoltageB!");
        return -1;
    }

    float actualVoltage = tpl0102_componentVars.highVoltage * (estPosf / 256);
    log_d("VoltageB set to %fV", actualVoltage);
    return 0;
}

int TPL0102::setDefaultVoltageA(float voltage){
    log_d("# Trying to set Default-VoltageA to %fV", voltage);

    if(voltage > tpl0102_componentVars.highVoltage){
        log_e("# Voltage target is higher that highVoltage!");
        log_e("# Failed to set Default-VoltageA!");
        return -1;
    }

    if(voltage < 0.0f){
        log_e("# Voltage target is negative!");
        log_e("# Failed to set Default-VoltageA!");
        return -1;
    }

    float estPosf = (voltage / tpl0102_componentVars.highVoltage) * 256;
    estPosf = std::round(estPosf);
    if(estPosf == 256) estPosf--;

    uint8_t estPos = static_cast<uint8_t>(estPosf);

    if(SetNonVolatileWiperA(estPos) == -1){
        log_e("# Failed to set Default-VoltageA!");
        return -1;
    }

    float actualVoltage = tpl0102_componentVars.highVoltage * (estPosf / 256);
    log_d("Default-VoltageA set to %fV", actualVoltage);
    return 0;
}

int TPL0102::setDefaultVoltageB(float voltage){
    log_d("# Trying to set Default-VoltageB to %fV", voltage);
    
    if(voltage > tpl0102_componentVars.highVoltage){
        log_e("# Voltage target is higher that highVoltage!");
        log_e("# Failed to set Default-VoltageB!");
        return -1;
    }

    if(voltage < 0.0f){
        log_e("# Voltage target is negative!");
        log_e("# Failed to set Default-VoltageB!");
        return -1;
    }

    float estPosf = (voltage / tpl0102_componentVars.highVoltage) * 256;
    estPosf = std::round(estPosf);
    if(estPosf == 256) estPosf--;

    uint8_t estPos = static_cast<uint8_t>(estPosf);

    if(SetNonVolatileWiperB(estPos) == -1){
        log_e("# Failed to set Default-VoltageB!");
        return -1;
    }

    float actualVoltage = tpl0102_componentVars.highVoltage * (estPosf / 256);
    log_d("# Default-VoltageB set to %fV", actualVoltage);
    return 0;
}


float TPL0102::getVoltageA(){
    float posf = static_cast<float>(tpl0102_componentVars.wiperPosA);

    float voltage = tpl0102_componentVars.highVoltage * (posf / 256);

    log_d("VoltageA is %fV", voltage);
    return voltage;
}

float TPL0102::getVoltageB(){
    float posf = static_cast<float>(tpl0102_componentVars.wiperPosB);

    float voltage = tpl0102_componentVars.highVoltage * (posf / 256);

    log_d("VoltageB is %fV", voltage);
    return voltage;
}

float TPL0102::getDefaultVoltageA(){
    float posf = static_cast<float>(tpl0102_componentVars.DefaultWiperPosA);

    float voltage = tpl0102_componentVars.highVoltage * (posf / 256);

    log_d("Default-VoltageA is %fV", voltage);
    return voltage;
}

float TPL0102::getDefaultVoltageB(){
    float posf = static_cast<float>(tpl0102_componentVars.DefaultWiperPosB);

    float voltage = tpl0102_componentVars.highVoltage * (posf / 256);

    log_d("Default-VoltageB is %fV", voltage);
    return voltage;
}


int TPL0102::enterShutdown(){
    log_d("# Trying to enter shutdown");
    
    if(canWriteAutoRetry()){
        uint8_t output = 0b10000000;

        log_d("# Entering Shutdown...");
        I2CTOOL::i2c_writeRegister(tpl0102_componentVars.I2C_ADDRESS, tpl0102_componentVars.REG_SETTINGS, output);
    }
    else{
        log_e("# Failed to enter shutdown!");
        return -1;
    }

    tpl0102_componentVars.shutdownEnabled = true;

    log_d("# Entered shutdown successfully.");
    return 0;
}


int TPL0102::exitShutdown(){
    log_d("# Trying to exit shutdown");
    
    if(canWriteAutoRetry()){
        uint8_t output = 0b11000000;

        log_d("Exiting Shutdown...");
        I2CTOOL::i2c_writeRegister(tpl0102_componentVars.I2C_ADDRESS, tpl0102_componentVars.REG_SETTINGS, output);
    }
    else{
        log_e("Failed to exit shutdown!");
        return -1;
    }

    tpl0102_componentVars.shutdownEnabled = false;

    log_d("Exited shutdown successfully.");
    return 0;
}

float TPL0102::getHighVoltage(){
    return tpl0102_componentVars.highVoltage;
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