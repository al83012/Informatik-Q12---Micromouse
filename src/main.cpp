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
#include "Components/vl53l4cd.h"
#include "measurement.h"
#include "driveControl.h"
#include "i2ctool.h"
#include "colors.h"
#include <VL53L4CD.h>

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
 /* DEBUG: PID-Controller:
 DRIVECONTROL::forward(30, 5);*/

//Measurement::IR::calibration::initCalibration(15);





//DRIVECONTROL::simpleForward1(20, 5);
//DRIVECONTROL::forward(20, 5);

}

void loop() {
 //Network::checkNetwork();
 delay(500);
 log_d(".");
 TMP464::DbgPrintTemperatures();
 //Measurement::Sensors::reportTemperature();
 DRV8424::debugPrintEncoderCounts();
//Measurement::IR::debugPrintRawDistance(1)
//Measurement::IR::debugPrintRawDistance(0);
//Measurement::IR::debugPrintRawDistance(2);
//Measurement::IR::debugPrintRawDistance(3);
//Measurement::IR::WallDetection::debugPrintWallDetectionStatus();
//TOF::debugReadAllSensors();






}