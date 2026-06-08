#ifndef esp32
#define esp32
#include <SPI.h>

struct Pins
{
    int PIN_SDA_0 = 48;
    int PIN_SCL_0 = 45;
    int PIN_SDA_1 = 36;
    int PIN_SCL_1 = 35;

    int PIN_LSM_CS = 16;   
    int PIN_MOSI = 15; 
    int PIN_MISO = 21;
    int PIN_SCLK = 14;

    int PIN_IR_LED_0 = 42;
    int PIN_IR_LED_1 = 1;
    int PIN_IR_LED_2 = 41;
    int PIN_IR_LED_3 = 2;

    int PIN_LSM_INT_1 = 43; //TXD0
    int PIN_LSM_INT_2 = 37;

    int PIN_FAN_EN = 12;
    int PIN_VL53_0_INT = 44; //RXD0
    int PIN_VL53_1_INT = 39;
    int PIN_VL53_2_INT = 40;

    int PIN_TCCAL_DRV_INT = 0;
    int PIN_IIS_INT = 38;
    int DRV_ENC1B = 46;
    int PIN_BQ_ALERT = 47;
    

};
extern Pins pins;

struct HardwareConfig
{
  int Serial_Clock = 115200;

};
extern HardwareConfig hardwareConfig;




class Esp32 {
public:
    static void initESP32();
    static void initSubComponents();
    static void initInterrupts();
    static void initPinStates();

    static Pins pins;
    static HardwareConfig hardwareConfig;

    static void shutdown();
   

};



#endif
