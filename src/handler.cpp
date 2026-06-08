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
   log_d("# CN OPENED");
  } else if (event == WebsocketsEvent::ConnectionClosed) {
    log_d("# CN CLOSED");
  } else if (event == WebsocketsEvent::GotPing) {
    log_i("# PONG"); 

    //client.pong(data);
  } else if (event == WebsocketsEvent::GotPong) {
    log_i("# PONG RCV!");
  }
}



void Handler::movePassive(int cells) {
    Simulation::sim_move(cells);  //REMOVE AFTER SIM
   log_d("# MV NO MSR > SRV");
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

      log_d("#RSLV NEW TASK");
      log_d(task.subStep);
      log_d(task.direction);
      log_d(task.reaction);

      if (task.subStep == sub_step || handleVars.MAX_SUB_STEPS) {
        int distance = measure(task.direction);
        log_d("Distance: ");
        log_d(distance);

        string content;
        content = "MEASUREMENT #" + to_string(globalVars.currCMD_ID) + " " + to_string(sub_step) + "_" + to_string(task.direction) + " " + to_string(distance);

        if (distance >= handleVars.SENSORLIMIT) { content = content + " SENSORLIMIT"; }
        log_d("#MSR > SRV");
        Utility::printClient(content);

        if (distance > handleVars.DISTANCE_THRESHOLD) {
          if (task.reaction == handleVars.STOP_IF_OPEN_ID) {
            string content = to_string(sub_step) + "_" + task.direction + "_STOP-IF-OPEN";
            log_d("#STOPPED: STOP-IF-OPEN");
            Utility::debug("Interrupt at substep " + sub_step);
            globalVars.interrupt = true;
            Utility::finishedAllInterrupt(content);
            return;
          }



        } else {


          if (task.direction == handleVars.STOP_IF_BLOCKED_ID) {
            string content = to_string(sub_step) + "_" + task.direction + "_STOP-IF-BLOCKED";
            log_d("#STOPPED: STOP_IF_BLOCKED");
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
  log_d("# TURN > SRV");
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
        log_d("#MSR > SRV");
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
  log_d(message.c_str());
  int words = 0;
  vector<MeasurementTask> activeTasks;

  vector<string> arguments = Utility::splitString(message, ' ');

  //DEBUG: Print arguments with id:
  for (int i = 0; i < arguments.size(); i++) {
    string content = to_string(i) + " -> \"" + arguments[i] + "\"";
    log_d(content.c_str());
  }
  
      


  if (!globalVars.desync_mode && !globalVars.wait_restart_confirm) {
    //Scanning for command type
    if (arguments[1].find("#") != string::npos) {
      log_d("# CMD RCV");


      arguments[1].erase(0, 1);
      globalVars.lastCMD_ID = globalVars.currCMD_ID;
      globalVars.currCMD_ID = stoi(arguments[1]);

      if (globalVars.lastCMD_ID == globalVars.currCMD_ID - 1) {
        log_d("# CMD_ID VALID:");
        log_d(globalVars.currCMD_ID);


        //Passive movement start
        if (arguments.size() == 3 || arguments[3] == "") {
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
                  log_e("# INVLD DIR ARGS");
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
              log_d(activeTasks.size());
              log_d("# ADD NEW TASK:");
              log_d(task.subStep);
              log_d(task.direction);
              log_d(task.reaction);
            }

            log_d("activeTasks:");
            for (int k = 0; k < activeTasks.size(); k++) {
              log_d(String(activeTasks[k].subStep));
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
            log_e("# INVLD MOV ARGS > SRV");
            Utility::debug("# INVLD MOV ARGS");
          }
        }


      } else {
        Utility::desync();
      }


    } else {
      log_e("# INVLD CMD-ID > SRV");
      Utility::debug("# INVLD CMD-ID");
    }

  } else {
    //Handle desync or await restart-confirm


    if(arguments[0] == "RESTART-CONFIRM") {
        log_i("# RSTRT CONFIRM RCV!");
        globalVars.wait_restart_confirm = false;
        return;
      }

      
  }
}