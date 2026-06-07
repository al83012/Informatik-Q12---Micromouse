#ifndef UTILITY_H
#define UTILITY_H

#include <string>
#include <vector>
using namespace std;


class Utility {
public:
    static void printClient(string message);
    static void stop();
    static void battery();
    static void restart();
    static void debug(string message);
    static void finishedAll();
    static void finishedAllInterrupt(string message);
    static void desync();    
    static std::vector<string> splitString(string str, char delimiter);
    
};


#endif