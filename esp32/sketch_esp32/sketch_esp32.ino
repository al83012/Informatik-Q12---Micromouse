
#include <ArduinoWebsockets.h>
#include "WiFi.h"
#include "HTTPClient.h"
#include <map>
#include <vector>

//Websocket
using namespace websockets;
using namespace std;
WebsocketsClient client;
String websockets_server = "ws://";
unsigned long lastReconnectAttempt = 0;
const unsigned long reconnectInterval = 2000;

//Network information
const char* ssid = "HOTSPOT-TEST";
const char* password = "012345678";
uint16_t port = 9001;
char serverName[] = "172.13.1.1";


//Simulation params
int SIM_SIZE = 8;
int cellSize = 18;
int stepFreq = 1;
int X = 1;
int Y = 1;
enum directions {
  posX,
  posY,
  negX,
  negY

};

enum directions dir = posX;


int SIM_FIELD[8][8] = {
  { 9, 1, 1, 1, 1, 1, 1, 1 },
  { 8, 0, 2, 8, 10, 0, 0, 2 },
  { 8, 0, 2, 8, 0, 0, 0, 2 },
  { 8, 0, 0, 0, 11, 5, 5, 2 },
  { 8, 0, 10, 8, 0, 0, 0, 2 },
  { 8, 0, 10, 8, 10, 8, 10, 10 },
  { 8, 0, 10, 8, 10, 8, 0, 2 },
  { 12, 4, 4, 4, 4, 4, 4, 6 }
};

//Live vars
bool desync_mode = false;
int lastCMD_ID = -1;
int currCMD_ID = -1;

struct MeasurementTask {
  int subStep;
  char direction;
  int reaction;
};

vector<MeasurementTask> activeTasks;

//Command config
int MAX_CMD_ARGS = 15;

int MAX_SUB_STEPS = 256;
int DISTANCE_THRESHOLD = 0;
int SENSORLIMIT = 5;


//IDs
int STOP_IF_OPEN_ID = 10;
int STOP_IF_BLOCKED_ID = 11;
int CONTINUE_ID = 12;

//---- Simulation methods start ----

void printField() {
  for (int i = 0; i < SIM_SIZE; i++) {
    for (int j = 0; j < SIM_SIZE; j++) {
      if (X == i & Y == j) {
        Serial.print("@");
      } else {
        Serial.print(SIM_FIELD[i][j]);
      }
    }
    Serial.println("");
  }
}

void sim_move(int n) {
  int newX = X;
  int newY = Y;
  int dir_flag;
  if (dir == posX) {
    newX += 1;
    dir_flag = 1;
  } else if (dir == posY) {
    newY += 1;
    dir_flag = 4;
  } else if (dir == negX) {
    newX -= 1;
    dir_flag = 8;
  } else if (dir == negY) {
    newY -= 1;
    dir_flag = 2;
  }

  if (can_move(newX, newY, dir_flag) && newX < SIM_SIZE && newY < SIM_SIZE) {
    X = newX;
    Y = newY;
  } else {
    Serial.println("# CANNOT MOVE");
  }
}

void sim_turn(int turns) {
  if (turns < 0) {
    turns = turns * -1;
    for (int i = 1; i <= turns; i++) {
      if (dir == posX) {
        dir = posY;
      } else if (dir == posY) {
        dir = negX;
      } else if (dir == negX) {
        dir = negY;
      } else if (dir == negY) {
        dir = posX;
      }
    }

  } else {
    for (int i = 1; i <= turns; i++) {
      if (dir == posX) {
        dir = negY;
      } else if (dir == negY) {
        dir = negX;
      } else if (dir == negX) {
        dir = posY;
      } else if (dir == posY) {
        dir = posX;
      }
    }
  }
}

int sim_measure(char scan_dir) {
  int distance = 0;
  int check_X = X;
  int check_Y = Y;
  directions lookDir = dir;



    if (scan_dir == 'L') {
    if (dir == posX) lookDir = posY;
    else if (dir == posY) lookDir = negX;
    else if (dir == negX) lookDir = negY;
    else if (dir == negY) lookDir = posX;
  }

  else if (scan_dir == 'R') {
    if (dir == posX) lookDir = negY;
    else if (dir == negY) lookDir = negX;
    else if (dir == negX) lookDir = posY;
    else if (dir == posY) lookDir = posX;
  }

 int wall_flag = 0;

 if (lookDir == posX) {
      wall_flag = 1;
    } else if (lookDir == posY) {
      wall_flag = 4;
    } else if (lookDir == negX) {
      wall_flag = 8;
    } else if (lookDir == negY) {
      wall_flag = 2;
    }

  while (true) {
   
    int next_X = check_X;
    int next_Y = check_Y;

  

    if (SIM_FIELD[check_Y][check_X] & wall_flag) { break; }
    if (distance >= SIM_SIZE) break;

    if (lookDir == posX) {
      next_X++;
    } else if (lookDir == posY) {
      next_Y++;
    } else if (lookDir == negX) {
      next_X--;
    } else if (lookDir == negY) {
      next_Y--;
    }

    if (next_X < 0 || next_X >= SIM_SIZE || next_Y < 0 || next_Y >= SIM_SIZE) { break; }

    check_X = next_X;
    check_Y = next_Y;
    distance++;
  }

  return distance;
}



bool can_move(int x, int y, int direction_flag) {
  if (SIM_FIELD[y][x] & direction_flag) {
    return false;
  }
  return true;
}




// ---- Simulation methods end ----




//---- Network methods start ----

String get_ws_url() {
  return websockets_server + WiFi.gatewayIP().toString() + ":" + port + "/";
}


void connectWS() {
  String ws_ip = get_ws_url();
  Serial.println("#Connecting to... " + ws_ip);

  client = WebsocketsClient();

  client.onMessage(handleCommand);
  client.onEvent(handleEvent);

  bool connected = client.connect(ws_ip);

  if (connected) {
    Serial.println("# CN SUCC!");
    client.send("DBG From the moment i understood the weakness of my flesh, it disgusted me...");
    client.ping();
  } else {
    Serial.println("# CN FAIL!");
  }
}


void initNetwork() {
  Serial.println("# INIT NTW CN...");
  WiFi.begin(ssid, password);
  Serial.println("# Connecting");
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }
  Serial.println("");
  Serial.println("# SUCC! MY IP: ");
  Serial.println(WiFi.localIP());

  WiFi.setAutoReconnect(true);
  WiFi.persistent(true);
  WiFi.setSleep(false);
}

void scanNetworks() {
  Serial.println("# SCAN FOR NW...");

  int n = WiFi.scanNetworks();
  Serial.println("# SCAN DONE!");
  if (n == 0) {
    Serial.println("# NO NW FOUND.");
  } else {
    Serial.println();
    Serial.print(n);
    Serial.println(" NW FOUND");
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
//---- Network methods end ----

void setup() {
  Serial.begin(115200);
  Serial.println("# ESP32 boot starting...");


  Serial.println("# Initializing WiFi...");
  WiFi.mode(WIFI_STA);
  scanNetworks();
  initNetwork();

  client = WebsocketsClient();

  client.onMessage(handleCommand);
  client.onEvent(handleEvent);
  connectWS();

  Serial.println("# Setup done!");

  /*handleCommand("MOVE #0 2$");
  handleCommand("MOVE #1 3$");
  handleCommand("MOVE #5 3$");
  handleCommand("MOVE #5 3$ Randomstuff");
  handleCommand("TURN #6 3$");
  handleCommand("RANDOMSTUFF");
  handleCommand("RANDOM #1 3$");*/
}


//---- Sensor-actor-methods start ----

int measure(char dir) {

  if (dir == 'F') {
    //read processed value of sensor -> return distance in walls
  }

  if (dir == 'R') {
    //read processed value of sensor -> return distance in walls
  }

  if (dir == 'L') {
    //read processed value of sensor -> return distance in walls
  }


  return sim_measure(dir);
}


void movePassive(int cells) {
  sim_move(cells);  //REMOVE AFTER SIM
  Serial.println("# MV NO MSR > SRV");
}


//TODO: Do not overwrite multiple measurements and allow multiple measurements per substep

void moveActive(int cells) {


  for (int i = 0; i < cells; i++) {
    int sub_step = i;
    bool anyStep = false;

    for(auto task : activeTasks) {
      if(task.subStep == sub_step || MAX_SUB_STEPS) {
        int distance = measure(task.direction);
        String content;
        if(task.subStep == MAX_SUB_STEPS) {
          content = String("MEASUREMENT #") + currCMD_ID + " " + String(sub_step) + "_" + task.direction + " " + String(distance);
        } else {
          content = String("MEASUREMENT #") + currCMD_ID + " " + String(sub_step) + "_" + task.direction + " " + String(distance);
        }
        if (distance >= SENSORLIMIT) { content = content + String(" SENSORLIMIT"); }
        Serial.println("#MSR > SRV");
        client.send(content);

         if (distance > DISTANCE_THRESHOLD) {
        if (task.reaction == STOP_IF_OPEN_ID) {
          debug("Interrupt at substep " + sub_step);  //TODO: Attach interrupt to finished message
          return;
        }

        if (task.reaction != CONTINUE_ID) {
          turnPassive(1);
          return;
        }

      } else if (distance <= DISTANCE_THRESHOLD) {


        if (task.direction == STOP_IF_BLOCKED_ID) {
          debug("Interrupt at substep " + sub_step);  //TODO: Attach interrupt to finished message
          return;
        }

        if (task.direction != CONTINUE_ID) {
          turnPassive(1);
          return;
        }
      }
        
      }
    }

    movePassive(1);
  
}
}

void turnPassive(int turns) {
  /* if(turns < 0) {
    turns = turns*-1;
    for(int i = 1; i <=turns; i++) {
      //turn counter-clockwise

  }


  } else {
    for(int i = 1; i <=turns; i++) {
      //turn clockwise
  }


  }*/
  sim_turn(turns);  //REMOVE AFTER SIM
}

void turnActive(int turns) {
  bool counterclock = false;
  if (turns < 0) {
    turns = turns * -1;
    counterclock = true;
  }

  for(int i = 0; i <= turns; i++) {
      int sub_step = i;
    for(auto task : activeTasks) {
      if(task.subStep == sub_step || MAX_SUB_STEPS) {
        int distance = measure(task.direction);
        String content;
        if(task.subStep == MAX_SUB_STEPS) {
          content = String("MEASUREMENT #") + currCMD_ID + " " + String(sub_step) + "_" + task.direction + " " + String(distance);
        } else {
          content = String("MEASUREMENT #") + currCMD_ID + " " + String(sub_step) + "_" + task.direction + " " + String(distance);
        }
        if (distance >= SENSORLIMIT) { content = content + String(" SENSORLIMIT"); }
        Serial.println("#MSR > SRV");
        client.send(content);

         if (distance > DISTANCE_THRESHOLD) {
        if (task.reaction == STOP_IF_OPEN_ID) {
          debug("Interrupt at substep " + sub_step);  //TODO: Attach interrupt to finished message
          return;
        }

        if (task.reaction != CONTINUE_ID) {
          turnPassive(1);
          return;
        }

      } else if (distance <= DISTANCE_THRESHOLD) {


        if (task.direction == STOP_IF_BLOCKED_ID) {
          debug("Interrupt at substep " + sub_step);  //TODO: Attach interrupt to finished message
          return;
        }

        if (task.direction != CONTINUE_ID) {
          turnPassive(1);
          return;
        }
      }
        
      }

      
      if(counterclock) {
        turnPassive(-1);
      } else {
        turnPassive(1);
      }
    }
}

}



//---- Sensor-actor-methods end ----



//---- Status messages start ----

void stop() {
  Serial.println("# STP > SRV");
  client.send("STOP");
}

void battery() {
  Serial.println("# BTRY > SRV");
  client.send("BATTERY 0");
}

void restart() {
  Serial.println("# RSTRT > SRV");

  //measurements.clear();
  //reactions.clear();
  lastCMD_ID = -1;
  currCMD_ID = -1;

  client.send("RESTART");
}

void debug(String message) {
  String content = "DBG " + message + "\n";
  client.send(content);
}

void finishedAll() {
  String message = "CMD-FINISHED #";
  Serial.println("# CMD DONE > SRV");
  Serial.println(String(X));
  Serial.println(String(Y));
  Serial.print(dir);


  message = message + currCMD_ID;
  client.send(message);
}


void desync() {
  Serial.println("# DSYNC > SRV");
  String error = "DESYNC ";
  for (int i = lastCMD_ID + 1; i < currCMD_ID; i++) {
    error = error + "#";
    error = error + i;
    error = error + " ";
  }
  client.send(error);
  Serial.println("# AWAIT RESYNC...");
  desync_mode = true;
}

//---- Status messages end ----

//---- Message-handling start ----

void handleEvent(WebsocketsEvent event, String data) {
  if (event == WebsocketsEvent::ConnectionOpened) {
    Serial.println("# CN OPENED");
  } else if (event == WebsocketsEvent::ConnectionClosed) {
    Serial.println("# CN CLOSED");
  } else if (event == WebsocketsEvent::GotPing) {
    Serial.println("PONG " + data);

    //client.pong(data);
  } else if (event == WebsocketsEvent::GotPong) {
    Serial.println("# PONG RCV!");
  }
}



void handleCommand(WebsocketsMessage WSmessage) {
  String message = WSmessage.data();
  Serial.println(">> " + message);
  String arguments[MAX_CMD_ARGS];
  int words = 0;
  //Collect words
  int lastIndex = 0;
  for (int i = 0; i <= message.length(); i++) {
    if (message[i] == ' ' || i == message.length()) {
      if (words < MAX_CMD_ARGS) {
        arguments[words] = message.substring(lastIndex, i);
        words++;
      }

      lastIndex = i + 1;
    }
  }

  //DEBUG: Print arguments with id:
  for (int i = 0; i < words; i++) {
    String content = String(i) + " -> \"" + String(arguments[i] + "\"");
    Serial.println(content);
  }


  if (!desync_mode) {
    //Scanning for command type
    if (arguments[1].indexOf("#") != -1) {
      Serial.println("# CMD RCV");
      arguments[1].remove(0, 1);
      lastCMD_ID = currCMD_ID;
      currCMD_ID = arguments[1].toInt();

      if (lastCMD_ID == currCMD_ID - 1) {
        Serial.print("# CMD_ID VALID:");
        Serial.println(currCMD_ID);


        //Passive movement start
        if (words == 3 || arguments[3] == "") {
          if (arguments[0] == "MOVE") {
            movePassive(arguments[2].toInt());
            finishedAll();

          } else if (arguments[0] == "TURN") {
            turnPassive(arguments[2].toInt());
            finishedAll();
          }
        //Passive movement end


        } else { // if not passive -> must be active
          MeasurementTask task;

          int cells = arguments[2].toInt();
          if (arguments[3] == "MEASURE") {
            for (int j = 4; j < words; j++) {
              String word = arguments[j];
      
              int sub_step;
              if (word[0] != 'X') {
                int dirIndex = -1;
                for (int k = 0; k < word.length(); k++) {
                  if (word[k] == '_') {
                    dirIndex = k;
                    break;
                  }
                }


                if (dirIndex != -1) {  
                  
                  task.subStep = word.substring(0, dirIndex).toInt();
                  task.direction = word[dirIndex +1];

                } else {
                  Serial.println("# INVLD DIR ARGS");
                  debug("INVLD DIR ARGS");
                }

              } else {
                task.subStep = MAX_SUB_STEPS;
                task.direction = word[2];

              }
              char dir = word[2];
              task.direction = dir;


              if (word.indexOf("STOP-IF-OPEN") != -1) {
                task.reaction = STOP_IF_OPEN_ID;
              } else if (word.indexOf("STOP-IF-BLOCKED") != -1) {
                task.reaction = STOP_IF_BLOCKED_ID;

              } else if (word.indexOf("TURN-IF-BLOCKED") != -1) {
                //reactions[sub_step] = String(word[word.length() - 1]).toInt();
              } else if (word.indexOf("CONTINUE") != -1) {
                task.reaction = CONTINUE_ID;
              }
            }
            activeTasks.push_back(task);
            if (arguments[0] == "MOVE") {
              moveActive(cells);
            } else if (arguments[0] == "TURN") {
              turnActive(cells);
            }
            activeTasks.clear();
            //reactions.clear();
            //measurements.clear();


            finishedAll();  //TODO: Attach interrupt to finished message

          } else {
            Serial.println("# INVLD MOV ARGS > SRV");
            debug("# INVLD MOV ARGS");
          }
        }


      } else {
        desync();
      }


    } else {
      Serial.println("# INVLD CMD-ID > SRV");
      debug("# INVLD CMD-ID");
    }

  } else {
  }
}


//---- Message-handling end ----



void loop() {

  if (client.available()) {
    client.poll();
  }

  // delay(1000/stepFreq);
}
