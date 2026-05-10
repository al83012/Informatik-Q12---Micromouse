#include <ArduinoWebsockets.h>
#include <Arduino.h>
#include "WiFi.h"
#include "HTTPClient.h"

#include "master.h"
#include "utility.h"
#include "network.h"
#include "simulation.h"
#include "handler.h"

using namespace websockets;
using namespace std;

HandleVars handleVars;

void Handler::handleEvent(WebsocketsEvent event, String data) {
      if (event == WebsocketsEvent::ConnectionOpened) {
    Serial.println("# CN OPENED");
  } else if (event == WebsocketsEvent::ConnectionClosed) {
    Serial.println("# CN CLOSED");
  } else if (event == WebsocketsEvent::GotPing) {
    Serial.println("# PONG"); 

    //client.pong(data);
  } else if (event == WebsocketsEvent::GotPong) {
    Serial.println("# PONG RCV!");
  }
}



void Handler::movePassive(int cells) {
    Simulation::sim_move(cells);  //REMOVE AFTER SIM
   Serial.println("# MV NO MSR > SRV");
}

int Handler::measure(char dir) {
  if (dir == 'F') {
    //read processed value of sensor -> return distance in walls
  }

  if (dir == 'R') {
    //read processed value of sensor -> return distance in walls
  }

  if (dir == 'L') {
    //read processed value of sensor -> return distance in walls
  }


  return Simulation::sim_measure(dir);
}



void Handler::moveActive(int cells, vector<MeasurementTask>& activeTasks) {
    
  for (int i = 0; i < cells; i++) {
    int sub_step = i;
    for (int j = 0; j < activeTasks.size(); j++) {
      auto task = activeTasks[j];

      Serial.println("#RSLV NEW TASK");
      Serial.println(task.subStep);
      Serial.println(task.direction);
      Serial.println(task.reaction);

      if (task.subStep == sub_step || handleVars.MAX_SUB_STEPS) {
        int distance = measure(task.direction);
        Serial.print("Distance: ");
        Serial.println(distance);

        string content;
        content = "MEASUREMENT #" + to_string(globalVars.currCMD_ID) + " " + to_string(sub_step) + "_" + to_string(task.direction) + " " + to_string(distance);

        if (distance >= handleVars.SENSORLIMIT) { content = content + " SENSORLIMIT"; }
        Serial.println("#MSR > SRV");
        Utility::printClient(content);

        if (distance > handleVars.DISTANCE_THRESHOLD) {
          if (task.reaction == handleVars.STOP_IF_OPEN_ID) {
            string content = to_string(sub_step) + "_" + task.direction + "_STOP-IF-OPEN";
            Serial.println("#STOPPED: STOP-IF-OPEN");
            Utility::debug("Interrupt at substep " + sub_step);
            globalVars.interrupt = true;
            Utility::finishedAllInterrupt(content);
            return;
          }



        } else {


          if (task.direction == handleVars.STOP_IF_BLOCKED_ID) {
            string content = to_string(sub_step) + "_" + task.direction + "_STOP-IF-BLOCKED";
            Serial.println("#STOPPED: STOP_IF_BLOCKED");
            Utility::debug("Interrupt at substep " + sub_step);  //TODO: Attach interrupt to finished message
            globalVars.interrupt = true;
            Utility::finishedAllInterrupt(content);
            return;
          }
        }
      }
    }

    movePassive(1);
  }
}


void Handler::turnPassive(int turns) {
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
  Serial.println("# TURN > SRV");
  Simulation::sim_turn(turns);  //REMOVE AFTER SIM
}



void Handler::turnActive(int turns, vector<MeasurementTask>& activeTasks) {
     bool counterclock = false;
  if (turns < 0) {
    turns = turns * -1;
    counterclock = true;
  }

  for (int i = 0; i <= turns; i++) {
    int sub_step = i;
    for (auto task : activeTasks) {
      if (task.subStep == sub_step || handleVars.MAX_SUB_STEPS) {
        int distance = measure(task.direction);
        string content;
        content = "MEASUREMENT #" + to_string(globalVars.currCMD_ID) + " " + to_string(sub_step) + "_" + to_string(task.direction) + " "  + to_string(distance);

        if (distance >= handleVars.SENSORLIMIT) { content = content + string(" SENSORLIMIT"); }
        Serial.println("#MSR > SRV");
        Utility::printClient(content);


        if (distance > handleVars.DISTANCE_THRESHOLD) {
          if (task.reaction == handleVars.STOP_IF_OPEN_ID) {
            string content = to_string(sub_step) + "_" + task.direction + "_STOP-IF-OPEN";
            Utility::debug("Interrupt at substep " + sub_step);  //TODO: Attach interrupt to finished message
            globalVars.interrupt = true;
            Utility::finishedAllInterrupt(content);
            return;
          }



        } else {


          if (task.reaction == handleVars.STOP_IF_BLOCKED_ID) {
            string content = to_string(sub_step) + "_" + to_string(task.direction) + "_STOP-IF-BLOCKED";
            Utility::debug("Interrupt at substep " + sub_step);  //TODO: Attach interrupt to finished message
            globalVars.interrupt = true;
            Utility::finishedAllInterrupt(content);
            return;
          }
        }
      }


      if (counterclock) {
        turnPassive(-1);
      } else {
        turnPassive(1);
      }
    }
  }
}





void Handler::handleCommand(WebsocketsMessage WSmessage) {

  globalVars.dir = posY;



  string message = WSmessage.data().c_str();
  Serial.println(message.c_str());
  string arguments[handleVars.MAX_CMD_ARGS];
  int words = 0;
  vector<MeasurementTask> activeTasks;
  //Collect words
  int lastIndex = 0;
  for (int i = 0; i <= message.length(); i++) {
    if (to_string(message[i]) == " " || i == message.length()) {
      if (words < handleVars.MAX_CMD_ARGS) {
        arguments[words] = message.substr(lastIndex, i);
        words++;
      }

      lastIndex = i + 1;
    }
  }

  //DEBUG: Print arguments with id:
  for (int i = 0; i < words; i++) {
    string content = to_string(i) + " -> \"" + arguments[i] + "\"";
    Serial.println(content.c_str());
  }


  if (!globalVars.desync_mode) {
    //Scanning for command type
    if (arguments[1].find("#") != string::npos) {
      Serial.println("# CMD RCV");
      arguments[1].erase(0, 1);
      globalVars.lastCMD_ID = globalVars.currCMD_ID;
      globalVars.currCMD_ID = stoi(arguments[1]);

      if (globalVars.lastCMD_ID == globalVars.currCMD_ID - 1) {
        Serial.print("# CMD_ID VALID:");
        Serial.println(globalVars.currCMD_ID);


        //Passive movement start
        if (words == 3 || arguments[3] == "") {
          if (arguments[0] == "MOVE") {
            movePassive(stoi(arguments[2]));
            Utility::finishedAll();

          } else if (arguments[0] == "TURN") {
            turnPassive(stoi(arguments[2]));
            Utility::finishedAll();
          }
          //Passive movement end


        } else {  // if not passive -> must be active
          MeasurementTask task;

          int cells = stoi(arguments[2]);
          if (arguments[3] == "MEASURE") {
            for (int j = 4; j < words; j++) {
              string word = arguments[j];

              int sub_step;
              if (to_string(word[0]) != "X") {
                int dirIndex = -1;
                for (int k = 0; k < word.length(); k++) {
                  if (to_string(word[k]) == "_") {
                    dirIndex = k;
                    break;
                  }
                }


                if (dirIndex != -1) {

                  task.subStep = stoi(word.substr(0, dirIndex));
                  task.direction = word[dirIndex + 1];

                } else {
                  Serial.println("# INVLD DIR ARGS");
                  Utility::debug("INVLD DIR ARGS");
                }

              } else {
                task.subStep = handleVars.MAX_SUB_STEPS;
                task.direction = word[2];
              }
              char dir = word[2];
              task.direction = dir;


              if (word.find("STOP-IF-OPEN") != string::npos) {
                task.reaction = handleVars.STOP_IF_OPEN_ID;
              } else if (word.find("STOP-IF-BLOCKED") != string::npos) {
                task.reaction = handleVars.STOP_IF_BLOCKED_ID;

              } else if (word.find("TURN-IF-BLOCKED") != string::npos) {
                //reactions[sub_step] = String(word[word.length() - 1]).toInt();
              } else if (word.find("CONTINUE") != string::npos) {
                task.reaction = handleVars.CONTINUE_ID;
              }
              activeTasks.push_back(task);
              Serial.println(activeTasks.size());
              Serial.println("# ADD NEW TASK:");
              Serial.println(task.subStep);
              Serial.println(task.direction);
              Serial.println(task.reaction);
            }

            Serial.println("activeTasks:");
            for (int k = 0; k < activeTasks.size(); k++) {
              Serial.println(String(activeTasks[k].subStep));
            }

            if (arguments[0] == "MOVE") {
              moveActive(cells, activeTasks);
            } else if (arguments[0] == "TURN") {
              turnActive(cells, activeTasks);
            }
            activeTasks.clear();



            if (!globalVars.interrupt) {
              Utility::finishedAll();
              globalVars.interrupt = false;
            }

          } else {
            Serial.println("# INVLD MOV ARGS > SRV");
            Utility::debug("# INVLD MOV ARGS");
          }
        }


      } else {
        Utility::desync();
      }


    } else {
      Serial.println("# INVLD CMD-ID > SRV");
      Utility::debug("# INVLD CMD-ID");
    }

  } else {
  }
}