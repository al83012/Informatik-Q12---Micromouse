#ifndef esp32
#define esp32
#include <SPI.h>
namespace Esp32 {



namespace HardwareConfig
{
  inline int Serial_Clock = 115200;

}

    void initESP32();
    void initSubComponents();
    void initInterrupts();
    void initPinStates();
    void shutdown();

}

#endif
