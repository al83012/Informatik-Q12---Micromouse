#include <Adafruit_NeoPixel.h>

#define LED_PIN 48
#define NUM_LEDS 1

Adafruit_NeoPixel pixel(NUM_LEDS, LED_PIN, NEO_GRB + NEO_KHZ800);

void setup() {
  pixel.begin();
  pixel.clear();
  pixel.show();
}

void loop() {
  pixel.setPixelColor(0, pixel.Color(255, 0, 0)); // Red
  pixel.show();
  delay(1000);

  pixel.setPixelColor(0, pixel.Color(0, 255, 0)); // Green
  pixel.show();
  delay(1000);

  pixel.setPixelColor(0, pixel.Color(0, 0, 255)); // Blue
  pixel.show();
  delay(1000);
}