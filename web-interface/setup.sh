#!/bin/bash

#remove old node version from raspberry
sudo apt-get remove nodered -y
sudo apt-get remove nodejs nodejs-legacy -y
sudo apt-get remove npm

#install new version for architecture armv7
sudo curl -sL https://deb.nodesource.com/setup_16.x | bash -
sudo apt-get -y install nodejs
sudo apt-get install npm -y

node install web_backend/