#ifndef network
#define network
#include <String>
using namespace std;

class Network {
public:
    static string getWsUrl();
    static void connectWS();
    static void initNetwork();
    static void scanNetworks();
};


#endif