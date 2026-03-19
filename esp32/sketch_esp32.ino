#include "WiFi.h"
#include "HTTPClient.h"


bool wallLeft;
bool wallRight;
bool wallFront;
int cellSize = 18;
int  stepFreq = 1;
WiFiClient client;
int counter = 0;

int lastCMD_ID = -1;
int currCMD_ID = -1;





const char* ssid= "SSID";
const char* password="PASSWORD";

char serverName[] = "172.13.1.1";

WiFiServer server(10000);


void setup() {
  Serial.begin(115200);
  Serial.println("# ESP32 boot starting...");

  
  Serial.println("# Initializing WiFi...");
  WiFi.mode(WIFI_STA);
  scanNetworks();

  initNetwork();

  Serial.println("# Setup done!");
  connectToServer();

  handleCommand("MOVE #0 2$");
  handleCommand("MOVE #1 3$");
  handleCommand("MOVE #5 3$");
  handleCommand("MOVE #5 3$ Randomstuff");
  handleCommand("TURN #6 3$");
  handleCommand("ALIVE 41$");
  handleCommand("RANDOMSTUFF");
  handleCommand("RANDOM #1 3$");


}


void forward(int cells, bool interruptOnSpace) {

  for(int i = 0; i < cells; i++) {
    if(interruptOnSpace) {

    } else {

    }
  }

}



void initNetwork() {
  Serial.println("# Initializing network connection...");
  WiFi.begin(ssid, password);
  Serial.println("# Connecting");
  while(WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }
  Serial.println("");
  Serial.println("# Success! My IP is: ");
  Serial.println(WiFi.localIP());
  server.begin();

}

void scanNetworks() {
  Serial.println("# Scanning for networks...");

  int n = WiFi.scanNetworks();
  Serial.println("# Scan done!");
  if (n == 0) {
    Serial.println("# No networks found.");
  } else {
    Serial.println();
    Serial.print(n);
    Serial.println(" networks found");
    for (int i = 0; i < n; ++i) {
      Serial.print(i + 1);
      Serial.print(": ");
      Serial.print(WiFi.SSID(i));
      Serial.print(" (");
      Serial.print(WiFi.RSSI(i));
      Serial.print(")");
      Serial.println((WiFi.encryptionType(i) == WIFI_AUTH_OPEN) ? " " : "*");
      delay(10);
    }
  }
  Serial.println("");
}



void connectToServer() {
   Serial.println("# Trying to connect to websocket...");

  if(!client.connect(serverName, 9001)) {
    Serial.println("# Connection to host failed");
    delay(1000);
    return;
  }

   debug("Hello from ESP32! Praise the Omnissiah!");
   debug("From the moment i understood the weakness of my flesh, it disgusted me");
   debug("For the machine is immortal");


}

void moveNoMeasure(int cells) {
  Serial.println("# MV NO MSR > SRV");
  finishedAll();
}

void moveMeasurePassive(int cells) {
  String content = "MEASUREMENT #" + currCMD_ID;
  content = content +  + " S_D DISTANCE$";
  Serial.println("# MOV MSR > SRV");
  client.print(content);

  finishedAll();
}

void turn() {
  finishedAll();
}

void alive(String message) {
  Serial.println("# CNF_ALV > SRV");
  client.print("CONFIRM-" + message);

}

void stop() {
  Serial.println("# STP > SRV");
  client.print("STOP$");
}

void battery() {
  Serial.println("# BTRY > SRV");
  client.print("BATTERY 0$");
}

void restart() {
  Serial.println("# RSTRT > SRV");
  client.print("RESTART$");
}

void debug(String message) {
  String content = "DBG " + message + "\n$";
  client.print(content);
}

void finishedAll(){
  String message = "# CMD-FINISHED #";
  Serial.println("# DONE > SRV");
  message = message + currCMD_ID;
  message = message + "$";
  client.print(message);

}


void desync() { 
  Serial.println("# DSYNC > SRV");
  String error = "DESYNC ";
  for(int i = lastCMD_ID+1; i < currCMD_ID; i++) {
      error = error + "#";
      error = error + i;
      error = error + " ";
      if(i == currCMD_ID-1) {
          error = error+"$";
      }

  }
  Serial.println(error); //sts
  Serial.println("# AWAIT RCV...KEEP ALV");


}

void handleCommand(String message) {

  Serial.println(">> " + message);
  message.remove(message.length()-1, 1);
  String arguments[15]; 
  int words = 0;

    int lastIndex = 0;
    for (int i = 0; i <= message.length(); i++) {
      if (message[i] == ' ' || i == message.length()) {
        if (words < 15) { 
          arguments[words] = message.substring(lastIndex, i);
          words++;
        }

      lastIndex = i + 1;
    }
  }
  
  for (int i = 0; i < words; i++) {

    

    if(arguments[i].indexOf("#") != -1) {
        Serial.println("# CMD RCV");
        arguments[i].remove(0, 1);
        lastCMD_ID = currCMD_ID;
        currCMD_ID = arguments[i].toInt();

        if(lastCMD_ID == currCMD_ID-1) {
            Serial.print("# CMD_ID VALID:");
            Serial.println(currCMD_ID);

            if(arguments[0] == "MOVE") {
                int cells = arguments[2].toInt();
                if(words == 3) {
                    moveNoMeasure(cells);
                } else {

                }          
                

            } else if (arguments[0] == "TURN") {
                turn();
            } else {
              Serial.println("UNKWN CMD!!");
              debug("UNKWN CMD!!");
            } 

            

            


        } else {
          desync();
        }


    } else {
      if(arguments[i].indexOf("ALIVE") != -1) {
          alive(message);
       }

    }
    
   // Serial.print(i);
   // Serial.println(": " + arguments[i]);

  }
  
}



void executeClient() {

 counter++;
  if(client.connected()) {
   

    char c = client.read();
    String message = "";

    while(c != '$' && client.connected()) {
      message+=c;
     c = client.read();
  }


    Serial.print("# RCV < SRV:");
    Serial.println(message);
    Serial.println("# Handling message...");

  } else {
    Serial.println("# Connection lost! Try reconnecting...");
    client.connect(serverName, 9001);
  }
  
}

void loop() {

  executeClient();
  delay(1000/stepFreq);

}
