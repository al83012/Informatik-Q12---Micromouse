#ifndef UTILITY
#define UTILITY
#include <string>
using namespace std;


class Utility {
public:
    static void printClient(string message);
    static void stop();
    static void battery();
    static void restart();
    static void debug(string message);
    static void finishedAll();
};


#endif