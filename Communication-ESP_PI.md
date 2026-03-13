Generell
- WiFi host = Raspberry Pi, kein externes WiFi
- MQTT broker = Mosquitto auf dem Pi
- Pi selbst --> Hostet zwar broker, ist aber auch Client
- Keine Nachricht darf auf das ankommen von Nachrichten in anderen Kanälen angewiesen sein
- QoS 2, um Ordnung innerhalb von Kanälen zu garantieren, und Befehlsanzahl einzuhalten --> Aber: Höhere Latenz

Kanäle
- Von Pi zu ESP32 (ESP = Subscriber)
  - conn_esp_pi/cmd --> Befehle
- Von ESP32 zu Pi (Pi = Subscriber)
  - conn_esp_pi/dbg --> Für Debug-prints
  - conn_esp_pi/battery --> Für Batteriestand-Meldung
  - conn_esp_pi/err --> Für Fehler-Meldung
  - conn_esp_pi/measure --> Für verarbeitete Abstandsmessungen
  - conn_esp_pi/complete --> Für Befehls-Vervollständigungs-Meldung
 
Raspberry Pi
- Mosquitto
  - Installation
  - Auto-startup --> Std Port 1883 --> unverschlüsselt reicht
- hostapd --> Pi als Access Point konfigurieren
  https://raspberrypi.stackexchange.com/questions/88214/setting-up-a-raspberry-pi-as-an-access-point-the-easy-way/88234#88234
- dhcpcd
  - .conf --> Statische ip-adresse für Host setzen --> Normalerweise 192.168.4.1
  - .conf --> DHCP für wlan0 eingebaute Schnittstelle hinzufügen
- dnsmasq
  - .conf --> Bearbeitet IP-Addresszuweisung für wlan0
  - .conf --> Address-Bereich für verbundene Geräte definieren
- Restart: hostapd und dnsmasq
- Boot (in Reihenfolge):
  - hostapd --> Startet WiFi
  - dnsmasq --> Startet DHCP
  - mosquitto --> Startet MQTT broker
  - --> Alles bereit für Verbindung mit ESP32
  - Subscriptions
  - Falls Maus noch nicht an ist --> Commands gehen ins Leere --> Muss vermieden werden
    - Sobald der ESP32 aktiv / verbunden ist muss eine Startup-Nachricht geschickt werden
- Separate Receive und Control Threads --> Messages sollen nicht blocken --> Kommunikation über Channels

ESP32
- Netzwerk-namen konfigurieren, auto-connect, etc
- IP 192.168.4.x
- Kann Broker über konst. Adresse finden --> ist bei 192.168.4.1
- Boot (in Reihenfolge):
  - Mit WiFi verbinden
  - Mit Broker verbinden
  - Subscriptions --> Vllt. mit Statusleuchte für verschiedene Verbindungs-zustände
  - Falls Broker noch nicht aktiv: In loop warten, bis das der Fall ist --> Statusleuchte
