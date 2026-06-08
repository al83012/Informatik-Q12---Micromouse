#ifndef SPITOOL_H
#define SPITOOL_H
#include "SPI.h"

struct SpiConfig
{
    int SPI_Clock = 1000000;
    SPISettings spiSettings;
    SpiConfig() : spiSettings(SPI_Clock, 1, 0) {}
};


class SPITOOL {
public:
    static SpiConfig spiConfig;
    static void init();
    static void spi_writeRegister(uint8_t registerAddress, uint8_t value, int PIN);
    static uint8_t spi_readRegister(uint8_t registerAddress, int PIN);

};



#endif