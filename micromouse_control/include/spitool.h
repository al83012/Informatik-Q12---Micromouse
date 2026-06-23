#ifndef SPITOOL_H
#define SPITOOL_H
#include "SPI.h"

namespace SPITOOL {
namespace Config {
    constexpr int SPI_Clock = 1000000;
    static const SPISettings spiSettings{SPI_Clock, 1, 0};

}
     void init();
     void spi_writeRegister(uint8_t registerAddress, uint8_t value, int PIN);
     uint8_t spi_readRegister(uint8_t registerAddress, int PIN);




}
#endif