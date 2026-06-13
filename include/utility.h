#ifndef UTILITY_H
#define UTILITY_H

#include <string>
#include <vector>
using namespace std;

namespace Utility {

    void printClient(string message);
    void stop();
    void battery();
    void restart();
    void debug(string message);
    void finishedAll();
    void finishedAllInterrupt(string message);
    void desync();    
    std::vector<string> splitString(string str, char delimiter);
    


}

#endif