#ifndef Pins_Arduino_h
#define Pins_Arduino_h

#include <stdint.h>
#include "soc/soc_caps.h"

#define USB_VID 0x303a
#define USB_PID 0x1001


/*
// Some boards have too low voltage on this pin (board design bug)
// Use different pin with 3V and connect with 48
// and change this setup for the chosen pin (for example 38)
#define PIN_NEOPIXEL        48
// BUILTIN_LED can be used in new Arduino API digitalWrite() like in Blink.ino
static const uint8_t LED_BUILTIN = SOC_GPIO_PIN_COUNT+PIN_NEOPIXEL;
#define BUILTIN_LED  LED_BUILTIN // backward compatibility
#define LED_BUILTIN LED_BUILTIN  // allow testing #ifdef LED_BUILTIN
// RGB_BUILTIN and RGB_BRIGHTNESS can be used in new Arduino API neopixelWrite()
#define RGB_BUILTIN LED_BUILTIN
#define RGB_BRIGHTNESS 64


static const uint8_t TX = 43;
static const uint8_t RX = 44;
*/

// DO NOT USE!
static const uint8_t SDA = 8;
// DO NOT USE!
static const uint8_t SCL = 9;

// DO NOT USE!
static const uint8_t SS    = 10;
//static const uint8_t MOSI  = 11;
//static const uint8_t MISO  = 13;

// DO NOT USE!
static const uint8_t SCK   = 12;

/*
static const uint8_t A0 = 1;
static const uint8_t A1 = 2;
static const uint8_t A2 = 3;
static const uint8_t A3 = 4;
static const uint8_t A4 = 5;
static const uint8_t A5 = 6;
static const uint8_t A6 = 7;
static const uint8_t A7 = 8;
static const uint8_t A8 = 9;
static const uint8_t A9 = 10;
static const uint8_t A10 = 11;
static const uint8_t A11 = 12;
static const uint8_t A12 = 13;
static const uint8_t A13 = 14;
static const uint8_t A14 = 15;
static const uint8_t A15 = 16;
static const uint8_t A16 = 17;
static const uint8_t A17 = 18;
static const uint8_t A18 = 19;
static const uint8_t A19 = 20;

static const uint8_t T1 = 1;
static const uint8_t T2 = 2;
static const uint8_t T3 = 3;
static const uint8_t T4 = 4;
static const uint8_t T5 = 5;
static const uint8_t T6 = 6;
static const uint8_t T7 = 7;
static const uint8_t T8 = 8;
static const uint8_t T9 = 9;
static const uint8_t T10 = 10;
static const uint8_t T11 = 11;
static const uint8_t T12 = 12;
static const uint8_t T13 = 13;
static const uint8_t T14 = 14;
*/



// USB: D-,
// Used for programming
// Do not touch!
static const uint8_t USB_DN = 19;
// USB: D+,
// Used for programming
// Do not touch!
static const uint8_t USB_DP = 20;


// I2C_0-Bus: SCL
// Fast I2C
// Clock frequency can be up to 1 MHz (in theory)
static const uint8_t SCL0 = 45;
// I2C_0-Bus: SDA
// Fast I2C
static const uint8_t SDA0 = 48;


// I2C_0-Bus: SCL
// Slow I2C
// Clock frequency can be up to 400 kHz (in theory)
static const uint8_t SCL1 = 35;
// I2C_0-Bus: SDA
// Slow I2C
static const uint8_t SDA1 = 36;


// SPI-Bus: MOSI
static const uint8_t MOSI = 15;
// SPI-Bus: MISO
static const uint8_t MISO = 21;
// SPI-Bus: SCLK
// Clock frequency can be up to 10 MHz (in theory)
static const uint8_t SCLK = 14;
// SPI-Bus: CS
static const uint8_t LSM_CS = 16;



// Interrupt
static const uint8_t BQ_INT = 47;



// Interrupt
static const uint8_t TCAL_DRV_INT = 0;

// PWM
static const uint8_t FAN_EN = 12;

// PWM
static const uint8_t DRV_AIN1 = 17;
// PWM
static const uint8_t DRV_AIN2 = 11;
// PWM
static const uint8_t DRV_BIN1 = 18;
// PWM
static const uint8_t DRV_BIN2 = 10;

// Interrupt
static const uint8_t ENC_A1 = 3;
// Interrupt
static const uint8_t ENC_A2 = 46;
// Interrupt
static const uint8_t ENC_B1 = 8;
// Interrupt
static const uint8_t ENC_B2 = 9;



// Interrupt
static const uint8_t VL_0_INT = 43;
// Interrupt
static const uint8_t VL_1_INT = 39;
// Interrupt
static const uint8_t VL_2_INT = 40;

// Interrupt
static const uint8_t LSM_INT_1 = 44;
// Interrupt
static const uint8_t LSM_INT_2 = 37;

// Interrupt
static const uint8_t IIS_INT = 38;

// Analog
static const uint8_t PD_0 = 7;
// Analog
static const uint8_t PD_1 = 6;
// Analog
static const uint8_t PD_2 = 5;
// Analog
static const uint8_t PD_3 = 4;

// MOSFET Gate
static const uint8_t IRLED_0 = 42;
// MOSFET Gate
static const uint8_t IRLED_1 = 1;
// MOSFET Gate
static const uint8_t IRLED_2 = 41;
// MOSFET Gate
static const uint8_t IRLED_3 = 2;

// GPIO
// Free tu use
static const uint8_t ESP_IO13 = 13;



#endif /* Pins_Arduino_h */