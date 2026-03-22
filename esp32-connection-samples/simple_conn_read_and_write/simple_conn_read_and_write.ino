#include <ArduinoWebsockets.h>
#include <WiFi.h>

using namespace websockets;

String websockets_server = "ws://";
uint16_t port = 9001;

WebsocketsClient client;

unsigned long lastReconnectAttempt = 0;
const unsigned long reconnectInterval = 2000;

void onMessageCallback(WebsocketsMessage message) {
  Serial.print("Got Message: ");
  Serial.println(message.data());
}

void onEventsCallback(WebsocketsEvent event, String data) {
  if (event == WebsocketsEvent::ConnectionOpened) {
    Serial.println("Connection Opened");
  } else if (event == WebsocketsEvent::ConnectionClosed) {
    Serial.println("Connection Closed");
  } else if (event == WebsocketsEvent::GotPing) {
    Serial.println("Ping " + data);
    
    //client.pong(data);
  } else if (event == WebsocketsEvent::GotPong) {
    Serial.println("Got a Pong!");
  }
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
  } else {
    Serial.println("Connection failed");
  }
}

void setup() {
  Serial.begin(115200);
  initWiFi();
  pinMode(RGB_BUILTIN, OUTPUT);
  

  client = WebsocketsClient();

  client.onMessage(onMessageCallback);
  client.onEvent(onEventsCallback);

  connectWS();
}

static unsigned long lastPing = 0;

void loop() {
  // Keep websocket alive
  client.poll();

  digitalWrite(RGB_BUILTIN, HIGH);

  /*if (millis() - lastPing > 1000 && client.available()) {
    client.ping(String(millis()));
    lastPing = millis();
  }*/

  if (!client.available()) {
    unsigned long now = millis();

    if (now - lastReconnectAttempt > reconnectInterval) {
      lastReconnectAttempt = now;

      Serial.println("Attempting reconnect...");
      connectWS();
    }
  }
}