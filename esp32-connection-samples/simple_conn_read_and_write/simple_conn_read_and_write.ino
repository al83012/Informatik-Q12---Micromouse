#include <ArduinoWebsockets.h>
#include <WiFi.h>

#include <Adafruit_NeoPixel.h>

#define LED_PIN 48
#define NUM_LEDS 1

#define C_SEARCH_WIFI pixel.Color(0, 100, 200)
#define C_WIFI_FOUND pixel.Color(0, 255, 255)
#define C_CONNECT pixel.Color(255, 255, 0)
#define C_CONNECT_FAIL pixel.Color(255, 200, 0)
#define C_CLOSED pixel.Color(255, 0, 0)
#define C_PING pixel.Color(0, 0, 255)
#define C_CONNECT_OPEN pixel.Color(0, 255, 0)
#define C_OFF pixel.Color(0, 0, 0)
#define C_MSG pixel.Color(255, 255, 255)


Adafruit_NeoPixel pixel(NUM_LEDS, LED_PIN, NEO_GRB + NEO_KHZ800);

using namespace websockets;




String wifi_name = "micromouse_pi_hotspot";
String wifi_password = "012345678";
String websockets_server = "ws://";
uint16_t port = 9001;

WebsocketsClient client;

unsigned long lastReconnectAttempt = 0;
const unsigned long reconnectInterval = 2000;

void onMessageCallback(WebsocketsMessage message) {
  pixel.setPixelColor(0, C_MSG);
  pixel.show();
  Serial.print("Got Message: ");
  Serial.println(message.data());
  delay(50);
  pixel.setPixelColor(0, C_OFF);
  pixel.show();
}

void onEventsCallback(WebsocketsEvent event, String data) {
  if (event == WebsocketsEvent::ConnectionOpened) {
    pixel.setPixelColor(0, C_CONNECT_OPEN);
    pixel.show();
    Serial.println("Connection Opened");
  } else if (event == WebsocketsEvent::ConnectionClosed) {
    pixel.setPixelColor(0, C_CLOSED);
    pixel.show();
    Serial.println("Connection Closed");
  } else if (event == WebsocketsEvent::GotPing) {
    Serial.println("Ping " + data);
    pixel.setPixelColor(0, C_PING);
    pixel.show();
    
    delay(100);
    pixel.setPixelColor(0, C_OFF);
    pixel.show();
    
    //client.pong(data);
  } else if (event == WebsocketsEvent::GotPong) {
    Serial.println("Got a Pong!");
  }
}

void initWiFi() {
  WiFi.mode(WIFI_STA);
  WiFi.begin(wifi_name, wifi_password);

  Serial.print("Connecting to WiFi ..");
  while (WiFi.status() != WL_CONNECTED) {
    Serial.print('.');
    pixel.setPixelColor(0, C_SEARCH_WIFI);
    pixel.show();
    delay(200);
    pixel.setPixelColor(0, C_OFF);
    pixel.show();
    delay(800);
  }

  pixel.setPixelColor(0, C_WIFI_FOUND);
  pixel.show();

  WiFi.setAutoReconnect(true);
  WiFi.persistent(true);
  WiFi.setSleep(false);

  Serial.println("\nConnected!");
  Serial.println(WiFi.localIP());
}

String get_ws_url() {
  return websockets_server + WiFi.gatewayIP().toString() + ":" + port + "/";
}

void connectWS() {
  String ws_ip = get_ws_url();
  Serial.println("Connecting to... " + ws_ip);

  client = WebsocketsClient();

  client.onMessage(onMessageCallback);
  client.onEvent(onEventsCallback);

  bool connected = client.connect(ws_ip);

  if (connected) {
    Serial.println("Connected!");
    client.send("Hi Server!");
    client.ping();
    pixel.setPixelColor(0, C_CONNECT);
    pixel.show();
  } else {
    pixel.setPixelColor(0, C_CONNECT_FAIL);
    pixel.show();
    delay(10);
    pixel.setPixelColor(0, C_OFF);
    pixel.show();
    Serial.println("Connection failed");
  }
}

void setup() {
  pixel.begin();
  pixel.clear();
  pixel.show();

  Serial.begin(115200);
  initWiFi();
  
  

  client = WebsocketsClient();

  client.onMessage(onMessageCallback);
  client.onEvent(onEventsCallback);


  connectWS();
}

static unsigned long lastPing = 0;

void loop() {
  // Keep websocket alive
  client.poll();


  /*if (millis() - lastPing > 1000 && client.available()) {
    client.ping(String(millis()));
    lastPing = millis();
  }*/

  if(!WiFi.isConnected()) {
    initWiFi();
  }

  if (!client.available()) {
    //pixel.setPixelColor(0, C_CONNECT_FAIL);
    unsigned long now = millis();

    if (now - lastReconnectAttempt > reconnectInterval) {
      lastReconnectAttempt = now;

      Serial.println("Attempting reconnect...");
      connectWS();
    }
  }
}
