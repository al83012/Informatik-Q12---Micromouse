#include "i2ctool.h"
#include "Components/Esp32.h"
#include "measurement.h"
#include  <iostream>

void I2CTOOL::init() {
    
    log_d("# Trying to initialize I2CTool");

    if(Wire.begin(SDA0, SCL0, I2CTOOL::Config::I2C_Clock_0)){
        log_i("# I2C0 initiated succesfully.");
    }
    else{
        log_e("# FAILED to initialize I2C0!");
    }

    if(Wire1.begin(SDA1, SCL1, I2CTOOL::Config::I2C_Clock_1)){
        log_i("# I2C1 initiated succesfully.");
    }
    else{
        log_e("# FAILED to initialize I2C1!");
    }
}



bool I2CTOOL::I2C0Write(uint8_t Address, uint8_t Register, uint8_t Data, bool AutoRetry){
    if(AutoRetry){
        log_d("# Trying to write on I2C0 with Auto-Retry...");

        for(int i = 0; i < I2CTOOL::Config::WRITERETRYATTEMPTS; i++){
            if(I2C0Write(Address, Register, Data, false)){
                log_d("# I2C0 Auto-Retry write success after %d tries.", ++i);
                return true;
            }

            delay(I2CTOOL::Config::WRITERETRYDELAY);
        }

        log_e("# I2C0 Auto-Retry write failed!");
        return false;
    }
    else{
        log_d("# Trying to write on I2C0...");

        Wire.beginTransmission(Address);

        Wire.write(Register);

        Wire.write(Data);

        uint8_t err = Wire.endTransmission(true);
        if (err != 0) {
            log_e("# I2C0 write failed with error code %d!", err);
            return false;
        }

        log_d("# I2C0 write success.");
        return true;

    }
}


bool I2CTOOL::I2C0Write(uint8_t Address, uint8_t Register, uint16_t Data, bool AutoRetry){
    if(AutoRetry){
        log_d("# Trying to write on I2C0 with Auto-Retry...");

        for(int i = 0; i < I2CTOOL::Config::WRITERETRYATTEMPTS; i++){
            if(I2C0Write(Address, Register, Data, false)){
                log_d("I2C0 Auto-Retry write success after %d tries.", ++i);
                return true;
            }

            delay(I2CTOOL::Config::WRITERETRYDELAY);
        }

        log_e("# I2C0 Auto-Retry write failed!");
        return false;
    }
    else{
        log_d("# Trying to write on I2C0...");

        Wire.beginTransmission(Address);

        Wire.write(Register);

        Wire.write((uint8_t)Data);
        Wire.write((uint8_t)(Data >> 8));

        uint8_t err = Wire.endTransmission(true);
        if (err != 0) {
            log_e("# I2C0 write failed with error code %d!", err);
            return false;
        }

        log_d("# I2C0 write success.");
        return true;

    }
}


bool I2CTOOL::I2C0Write(uint8_t Address, uint8_t Register, const uint8_t* DataStart, size_t Size, bool AutoRetry){
    if(AutoRetry){
        log_d("# Trying to write on I2C0 with Auto-Retry...");

        for(int i = 0; i < I2CTOOL::Config::WRITERETRYATTEMPTS; i++){
            if(I2C0Write(Address, Register, DataStart, Size, false)){
                log_d("I2C0 Auto-Retry write success after %d tries.", ++i);
                return true;
            }

            delay(I2CTOOL::Config::WRITERETRYDELAY);
        }

        log_e("# I2C0 Auto-Retry write failed!");
        return false;
    }
    else{
        log_d("# Trying to write on I2C0...");

        Wire.beginTransmission(Address);

        Wire.write(Register);

        Wire.write(DataStart, Size);

        uint8_t err = Wire.endTransmission(true);
        if (err != 0) {
            log_e("# I2C0 write failed with error code %d!", err);
            return false;
        }

        log_d("# I2C0 write success.");
        return true;

    }
}


bool I2CTOOL::I2C1Write(uint8_t Address, uint8_t Register, uint8_t Data, bool AutoRetry){
    if(AutoRetry){
        log_d("# Trying to write on I2C1 with Auto-Retry...");

        for(int i = 0; i < I2CTOOL::Config::WRITERETRYATTEMPTS; i++){
            if(I2C1Write(Address, Register, Data, false)){
                log_d("# I2C1 Auto-Retry write success after %d tries.", ++i);
                return true;
            }

            delay(I2CTOOL::Config::WRITERETRYDELAY);
        }

        log_e("# I2C1 Auto-Retry write failed!");
        return false;
    }
    else{
        log_d("# Trying to write on I2C1...");

        Wire1.beginTransmission(Address);

        Wire1.write(Register);

        Wire1.write(Data);

        uint8_t err = Wire1.endTransmission(true);
        if (err != 0) {
            log_e("# I2C1 write failed with error code %d!", err);
            return false;
        }

        log_d("# I2C1 write success.");
        return true;

    }
}


bool I2CTOOL::I2C1Write(uint8_t Address, uint8_t Register, uint16_t Data, bool AutoRetry){
    if(AutoRetry){
        log_d("# Trying to write on I2C1 with Auto-Retry...");

        for(int i = 0; i < I2CTOOL::Config::WRITERETRYATTEMPTS; i++){
            if(I2C1Write(Address, Register, Data, false)){
                log_d("# I2C1 Auto-Retry write success after %d tries.", ++i);
                return true;
            }

            delay(I2CTOOL::Config::WRITERETRYDELAY);
        }

        log_e("# I2C1 Auto-Retry write failed!");
        return false;
    }
    else{
        log_d("# Trying to write on I2C1...");

        Wire1.beginTransmission(Address);

        Wire1.write(Register);

        Wire1.write((uint8_t)Data);
        Wire1.write((uint8_t)(Data >> 8));

        uint8_t err = Wire1.endTransmission(true);
        if (err != 0) {
            log_e("# I2C1 write failed with error code %d!", err);
            return false;
        }

        log_d("# I2C1 write success.");
        return true;

    }
}

bool I2CTOOL::I2C1Write(uint8_t Address, uint8_t Register, const uint8_t* DataStart, size_t Size, bool AutoRetry){
    if(AutoRetry){
        log_d("# Trying to write on I2C1 with Auto-Retry...");

        for(int i = 0; i < I2CTOOL::Config::WRITERETRYATTEMPTS; i++){
            if(I2C1Write(Address, Register, DataStart, Size, false)){
                log_d("# I2C1 Auto-Retry write success after %d tries.", ++i);
                return true;
            }

            delay(I2CTOOL::Config::WRITERETRYDELAY);
        }

        log_e("# I2C1 Auto-Retry write failed!");
        return false;
    }
    else{
        log_d("# Trying to write on I2C1...");

        Wire1.beginTransmission(Address);

        Wire1.write(Register);

        Wire1.write(DataStart, Size);

        uint8_t err = Wire1.endTransmission(true);
        if (err != 0) {
            log_e("# I2C1 write failed with error code %d!", err);
            return false;
        }

        log_d("# I2C1 write success.");
        return true;

    }
}

bool I2CTOOL::I2C0Read(uint8_t Address, uint8_t Register, uint8_t& ReadOutput, bool AutoRetry){
    if(AutoRetry){
        log_d("# Trying to read on I2C0 with Auto-Retry...");

        for(int i = 0; i < I2CTOOL::Config::READRETRYATTEMPTS; i++){
            if(I2C0Read(Address, Register, ReadOutput, false)){
                log_d("# I2C0 Auto-Retry read success after %d tries.", ++i);
                return true;
            }

            delay(I2CTOOL::Config::READRETRYDELAY);
        }

        log_e("# I2C0 Auto-Retry read failed!");
        return false;
    }
    else{
        log_d("# Trying to read on I2C0...");

        Wire.beginTransmission(Address);

        Wire.write(Register);

        uint8_t err = Wire.endTransmission(false);
        if (err != 0) {
            log_e("# I2C0 read failed with error code %d!", err);
            return false;
        }

        uint8_t bytesReceived = Wire.requestFrom(Address, (uint8_t)1);
        if (bytesReceived != 1) {
            log_e("# I2C read failed as bytes-recieved doesn't match bytes-requested!");
            return false;
        }

        ReadOutput = Wire.read();

        log_d("# I2C0 read success.");
        return true;

    }
}

bool I2CTOOL::I2C0Read(uint8_t Address, uint8_t Register, uint16_t& ReadOutput, bool AutoRetry){
    if(AutoRetry){
        log_d("# Trying to read on I2C0 with Auto-Retry...");

        for(int i = 0; i < I2CTOOL::Config::READRETRYATTEMPTS; i++){
            if(I2C0Read(Address, Register, ReadOutput, false)){
                log_d("# I2C0 Auto-Retry read success after %d tries.", ++i);
                return true;
            }

            delay(I2CTOOL::Config::READRETRYDELAY);
        }

        log_e("# I2C0 Auto-Retry read failed!");
        return false;
    }
    else{
        log_d("# Trying to read on I2C0...");

        Wire.beginTransmission(Address);

        Wire.write(Register);

        uint8_t err = Wire.endTransmission(false);
        if (err != 0) {
            log_e("# I2C0 read failed with error code %d!", err);
            return false;
        }

        uint8_t bytesReceived = Wire.requestFrom(Address, (uint8_t)2);
        if (bytesReceived != 2) {
            log_e("# I2C read failed as bytes-recieved doesn't match bytes-requested!");
            return false;
        }

        uint8_t msb = Wire.read();
        uint8_t lsb = Wire.read();

        ReadOutput = (((uint16_t)msb) << 8) | lsb;

        log_d("# I2C0 read success.");
        return true;

    }
}


bool I2CTOOL::I2C0Read(uint8_t Address, uint8_t Register, uint8_t* ReadOutput, size_t Size, bool AutoRetry){
    if(AutoRetry){
        log_d("# Trying to read on I2C0 with Auto-Retry...");

        for(int i = 0; i < I2CTOOL::Config::READRETRYATTEMPTS; i++){
            if(I2C0Read(Address, Register, ReadOutput, Size, false)){
                log_d("# I2C0 Auto-Retry read success after %d tries.", ++i);
                return true;
            }

            delay(I2CTOOL::Config::READRETRYDELAY);
        }

        log_e("# I2C0 Auto-Retry read failed!");
        return false;
    }
    else{
        log_d("# Trying to read on I2C0...");

        Wire.beginTransmission(Address);

        Wire.write(Register);

        uint8_t err = Wire.endTransmission(false);
        if (err != 0) {
            log_e("# I2C0 read failed with error code %d!", err);
            return false;
        }

        uint8_t bytesReceived = Wire.requestFrom(Address, Size, true);
        if (bytesReceived != Size) {
            log_e("# I2C read failed as bytes-recieved doesn't match bytes-requested!");
            return false;
        }


        for(int i = 0; i < Size; i++){
            ReadOutput[i] = (uint8_t)Wire.read();
        }

        log_d("# I2C0 read success.");
        return true;

    }
}


bool I2CTOOL::I2C1Read(uint8_t Address, uint8_t Register, uint8_t& ReadOutput, bool AutoRetry){
    if(AutoRetry){
        log_d("# Trying to read on I2C1 with Auto-Retry...");

        for(int i = 0; i < I2CTOOL::Config::READRETRYATTEMPTS; i++){
            if(I2C1Read(Address, Register, ReadOutput, false)){
                log_d("I2C1 Auto-Retry read success after %d tries.", ++i);
                return true;
            }

            delay(I2CTOOL::Config::READRETRYDELAY);
        }

        log_e("# I2C1 Auto-Retry read failed!");
        return false;
    }
    else{
        log_d("# Trying to read on I2C1...");

        Wire1.beginTransmission(Address);

        Wire1.write(Register);

        uint8_t err = Wire1.endTransmission(false);
        if (err != 0) {
            log_e("# I2C1 read failed with error code %d!", err);
            return false;
        }

        uint8_t bytesReceived = Wire1.requestFrom(Address, (uint8_t)1);
        if (bytesReceived != 1) {
            log_e("# I2C read failed as bytes-recieved doesn't match bytes-requested!");
            return false;
        }

        ReadOutput = Wire1.read();

        log_d("# I2C1 read success.");
        return true;

    }
}


bool I2CTOOL::I2C1Read(uint8_t Address, uint8_t Register, uint16_t& ReadOutput, bool AutoRetry){
    if(AutoRetry){
        log_d("# Trying to read on I2C1 with Auto-Retry...");

        for(int i = 0; i < I2CTOOL::Config::READRETRYATTEMPTS; i++){
            if(I2C1Read(Address, Register, ReadOutput, false)){
                log_d("I2C1 Auto-Retry read success after %d tries.", ++i);
                return true;
            }

            delay(I2CTOOL::Config::READRETRYDELAY);
        }

        log_e("# I2C1 Auto-Retry read failed!");
        return false;
    }
    else{
        log_d("# Trying to read on I2C1...");

        Wire1.beginTransmission(Address);

        Wire1.write(Register);

        uint8_t err = Wire1.endTransmission(false);
        if (err != 0) {
            log_e("# I2C1 read failed with error code %d!", err);
            return false;
        }

        uint8_t bytesReceived = Wire1.requestFrom(Address, (uint8_t)2);
        if (bytesReceived != 2) {
            log_e("# I2C read failed as bytes-recieved doesn't match bytes-requested!");
            return false;
        }

        uint8_t msb = Wire1.read();
        uint8_t lsb = Wire1.read();

        ReadOutput = (((uint16_t)msb) << 8) | lsb;

        log_d("# I2C1 read success.");
        return true;

    }
}


bool I2CTOOL::I2C1Read(uint8_t Address, uint8_t Register, uint8_t* ReadOutput, size_t Size, bool AutoRetry){
    if(AutoRetry){
        log_d("# Trying to read on I2C1 with Auto-Retry...");

        for(int i = 0; i < I2CTOOL::Config::READRETRYATTEMPTS; i++){
            if(I2C1Read(Address, Register, ReadOutput, Size, false)){
                log_d("# I2C1 Auto-Retry read success after %d tries.", ++i);
                return true;
            }

            delay(I2CTOOL::Config::READRETRYDELAY);
        }

        log_e("# I2C1 Auto-Retry read failed!");
        return false;
    }
    else{
        log_d("# Trying to read on I2C1...");

        Wire1.beginTransmission(Address);

        Wire1.write(Register);

        uint8_t err = Wire1.endTransmission(false);
        if (err != 0) {
            log_e("# I2C1 read failed with error code %d!", err);
            return false;
        }

        uint8_t bytesReceived = Wire1.requestFrom(Address, Size, true);
        if (bytesReceived != Size) {
            log_e("# I2C read failed as bytes-recieved doesn't match bytes-requested!");
            return false;
        }


        for(int i = 0; i < Size; i++){
            ReadOutput[i] = (uint8_t)Wire1.read();
        }

        log_d("# I2C1 read success.");
        return true;

    }
}

void I2CTOOL::flip(uint16_t& Data){
    uint16_t tmp = Data;
    Data = (Data >> 8) | (tmp << 8);
}

void I2CTOOL::flip(uint8_t* Data, size_t Size){
    for(int i = 0; i < Size / 2; i++){
        std::swap(Data[i], Data[Size - i - 1]);
    }
}


#define I2C0Write55 I2CT.I2C0Write // possible maybe use different name than function


void I2CTOOL::I2CScanner(){
    uint8_t Devices0 = 0;
    uint8_t Devices1 = 0;
    uint8_t error;
    uint8_t address;

    log_i("Scanning for I2C devices...");
    log_i("Scanning I2C0...");
    for(address = 1; address < 127; address++){
        Wire.beginTransmission(address);
        error = Wire.endTransmission();

        if(error == 0){
            Devices0++;
            if(address < 16){
                log_i("I2C0 device found at address 0x0%X", address);
            }
            else{
                log_i("I2C0 device found at address 0x%X", address);
            }
        }
        else if(error == 4){
            if(address < 16){
                log_e("I2C0 error at address 0x0%X", address);
            }
            else{
                log_e("I2C0 error at address 0x%X", address);
            }
        }
    }
    log_i("I2C0 devices found: %d", Devices0);

    log_i("Scanning I2C1...");
    for(address = 1; address < 127; address++){
        Wire1.beginTransmission(address);
        error = Wire1.endTransmission();

        if(error == 0){
            Devices1++;
            if(address < 16){
                log_i("I2C1 device found at address 0x0%X", address);
            }
            else{
                log_i("I2C1 device found at address 0x%X", address);
            }
        }
        else if(error == 4){
            if(address < 16){
                log_e("I2C1 error at address 0x0%X", address);
            }
            else{
                log_e("I2C1 error at address 0x%X", address);
            }
        }
    }
    log_i("I2C1 devices found: %d", Devices1);
}

void I2CTOOL::findComponent(uint8_t I2C_ADDRESS, Measurement::Sensors::SensorNames SensorName) {
    using namespace Measurement::Sensors;

    Wire1.beginTransmission(I2C_ADDRESS);
    if (Wire1.endTransmission() != 0) {
        log_e("%s not found at address %d", Measurement::Sensors::to_string(SensorName), I2C_ADDRESS);
    } else {
        log_i("%s initialized successfully", Measurement::Sensors::to_string(SensorName));
    }
}

void I2CTOOL::findComponent(Measurement::Sensors::SensorNames SensorName) {
    using namespace Measurement::Sensors;

    Wire1.beginTransmission(Measurement::Sensors::getI2CAddress(SensorName));
    if (Wire1.endTransmission() != 0) {
        log_e("%s not found at address %d", Measurement::Sensors::to_string(SensorName), Measurement::Sensors::getI2CAddress(SensorName));
    } else {
        log_i("%s initialized successfully", Measurement::Sensors::to_string(SensorName));
    }
}
