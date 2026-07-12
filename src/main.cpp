#include "Arduino.h"
#include "WiFi.h"
#include "ArduinoWebsockets.h"
#include "HTTPClient.h"

#include "utility.h"
#include "master.h"
#include "network.h"
#include "handler.h"
#include "Components/Esp32.h"
#include "components/tpl0102.h"
#include "components/tmp464.h"
#include "measurement.h"
#include "Components/drv8424.h"
#include "measurement.h"
#include "driveControl.h"
#include "i2ctool.h"
#include "colors.h"

using namespace COLORS;


void setup() {
delay(1000);
log_i("# Initializing Setup");
Esp32::initESP32();
// Network::setup();

 I2CTOOL::I2CScanner();

 TPL0102::DbgPrintVoltages();
 TMP464::DbgPrintTemperatures(); 


 log_i(GREEN "# Setup done!");
   // DRIVECONTROL::forward(30, 15);
 DRV8424::setDutyCycle1(40);
 DRV8424::setDutyCycle2(40);

}

void loop() {
 //Network::checkNetwork();
 delay(2000);
 log_d(".");
 TMP464::DbgPrintTemperatures();
 //Measurement::Sensors::reportTemperature();
 DRV8424::debugPrintEncoderCounts();



/* 
DEBUG: TEST IR-SENSORS

Measurement::IR::refreshDistance(1);
Measurement::IR::debugPrintRawDistance(1);

Measurement::IR::refreshDistance(0);
Measurement::IR::debugPrintRawDistance(0);

Measurement::IR::refreshDistance(2);
Measurement::IR::debugPrintRawDistance(2);

Measurement::IR::refreshDistance(3);
Measurement::IR::debugPrintRawDistance(3);
*/



/*
DEBUG: THRESHOLD CALIBRATION
Measurement::IR::calibration::initCalibration(5);


DEBUG: TEST WALL DETECTION
Measurement::IR::WallDetection::debugPrintWallDetectionStatus();
*/






}