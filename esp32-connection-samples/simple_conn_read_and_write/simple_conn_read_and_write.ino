/*
  Example from WiFi > WiFiScan
  Complete details at https://RandomNerdTutorials.com/esp32-useful-wi-fi-functions-arduino/
*/

#include "WiFi.h"

void setup() {
  Serial.begin(115200);

  delay(3000);

  while (!Serial) {}

  Serial.println("BOOT OK");


  // Set WiFi to station mode and disconnect from an AP if it was previously connected
  WiFi.mode(WIFI_STA);
  WiFi.disconnect();
  delay(100);

  Serial.println("Setup done");

  initWiFi();
}

WiFiClient client;
uint16_t port = 9001;

void loop() {
  serverConnection();
}

void initWiFi() {
  WiFi.mode(WIFI_STA);
  WiFi.begin("HOTSPOT-TEST", "12345678");
  Serial.print("Connecting to WiFi ..");
  while (WiFi.status() != WL_CONNECTED) {
    Serial.print('.');
    delay(1000);
  }
  WiFi.setAutoReconnect(true);
  WiFi.persistent(true);
  Serial.println(WiFi.localIP());
  Serial.println(WiFi.gatewayIP());
}

int n = 0;
String str = "";

void serverConnection() {

  if (!client.connected()) {
    Serial.println("Not connected");
    client.connect(WiFi.gatewayIP(), port);

    return;
  }

  /*if (n % 1000 == 999) {
    client.stop();
    Serial.println("STOPPED");
    n += 1;
    return;
  }*/

  //Serial.println("CONN");
  String str = "ECHO: ";
  bool readMsg = false;
  while (client.available()) {
    char c = client.read();
    Serial.print("Read: '");
    Serial.print(c);
    Serial.println("'");
    str += c;
    if( c == '$') {
      if (str.indexOf("ALIVE") > 0) {
        Serial.println("CONFIRM-ALIVE$");
        client.print("CONFIRM-ALIVE$");
      } else {
        Serial.println(str);
        client.print(str);
      }
      str = "";
      return;
    }
  }
}
