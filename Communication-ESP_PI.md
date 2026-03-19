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
- Rust:
  - rumqttc https://github.com/bytebeamio/rumqtt/tree/main/rumqttc --> Async Client ! NEVER BLOCK POLLING --> NECESSARY FOR EVENT LOOP PROGRESS
  - Kann Verbindung über IP-Adresse aufbauen --> Muss nur zu Broker connecten --> 192.168.4.1

ESP32
- Netzwerk-namen konfigurieren, auto-connect, etc
- IP 192.168.4.x
- Kann Broker über konst. Adresse finden --> ist bei 192.168.4.1
- Boot (in Reihenfolge):
  - Mit WiFi verbinden
  - Mit Broker verbinden
  - Subscriptions --> Vllt. mit Statusleuchte für verschiedene Verbindungs-zustände
  - Falls Broker noch nicht aktiv: In loop warten, bis das der Fall ist --> Statusleuchte
 
### Kommunikationsprotokoll
- Es gibt keinen guten Weg, Verbindungsabbruch durch ausschalten des ESP32 festzustellen --> Man braucht periodische Nachrichten, die erkennen, wann die Verbindung abbricht
- Es ist nicht sicher, dass Nachrichten ankommen --> Alle Informationen, die für eine Nachricht wichtig sind müssen in dieser Nachricht sein
- Der PI muss sicher Nachrichten an den ESP32 schicken können andersherum ist es nicht so dramatisch --> Die Nachrichten des PI müssen nummeriert werden
- Der PI braucht acknowledgements für alle Bewegungen
- Separate Komponenten der Befehle sollten für die einfache Erkennbarkeit durch einfache leerzeichen getrennt werden
#### CMD (vom PI)
- <CMD_ID> ist ein u32
- Bewegungsbefehl: `MOVE #<CMD_ID> <N>$` --> N Felder Forwärts; N ist ein u8
- Bewegungsbefehl mit Messungen:
  `MOVE #<CMD_ID> <N> MEASURE <Measurement_Tasks>$`
- <Measurement_Task>s (Nur als Teil eines Bewegungs-Befehls):
  `<N>_<L/R/F>_<CONTINUE/STOP_IF_OPEN/STOP_IF_BLOCKED>` --> Beim Nten Teilschritt der Bewegung links, rechts oder forwärts Distanz prüfen und schicken; Falls Continue --> Einfach weiterfahren, sonst entweder stoppen, weil dort direkt eine Wand ist, oder, weil dort keine Wand ist
- Drehungsbefehl: `TURN #<CMD_ID> <X>$` --> 0 = Keine Drehung; X = Ganzzahl => Zahl * 90° links (neg. = rechts)

#### DBG (vom ESP32)
- `DBG <Nachricht>$` --> Wird an Frontend weitergeleitet, darf keine $ enthalten --> würden Nachricht frühzeitig beenden

#### MEASUREMENT (vom ESP32)
- Wird immer geschickt, bevor das zu einer Bewegung gehörende Bewegungs-Ack kommt
- <CMD_ID> ist derselbe u32, der vorher bei der Bewegungs-Anweisung geschickt wurde, die zu der Messung geführt hat
- `MEASUREMENT #<CMD_ID> <N>_<L/R/F> <DEPTH>$` --> DEPTH ist 0, falls direkt eine Wand ist, je 1 größer, wenn 1 Zelle weiter leer ist
- `MEASUREMENT #<CMD_ID> <N>_<L/R/F> <DEPTH> SENSORLIMIT$` --> Mindestens die Tiefe wurde erreicht, Sensor kann nicht zuverlässig weiter schauen

#### CMD_FINISHED (vom ESP32)
- <CMD_ID> wieder derselbe von davor
- CMD_FINISHED wird immer nach jedem MEASUREMENT geschickt, der von einem CMD verursacht wird
- `CMD_FINISHED #<CMD_ID>$` --> Befehl vollständig ausgeführt
- `CMD_FINISHED #<CMD_ID> <N>_<L/R/F>_<STOP_IF_OPEN/STOP_IF_BLOCKED>$` --> Frühzeitig beendeter Befehl + Begründung

#### DESYNC (vom ESP32)
- Geschickt, falls eine CMD_ID nicht in 1er-Schritten hochzählt --> eine Nachricht wurde nicht empfangen
- `DESYNC #<CMD_ID> #<CMD_ID> ...$` --> Gibt alle CMD_IDs an, die Übersprungen wurden
- Falls ein DESYNC passiert, soll der ESP32 den frühzeitig geschickten Befehl nicht ausführen und stattdessen darauf warten, dass die verlorenen Commands geschickt werden (ohne Garantie für Reihenfolge diesmal, soll einfach warten, bis alle von ihnen da sind)

#### ALIVE (vom PI)
- `ALIVE <Timestamp>$` --> Alle paar Sekunden --> sagt verbundenem ESP32, dass die Verbindung zum PI immer noch aktiv ist --> <Timestamp> ist eine Zeit in Sekunden seit start

#### CONFIRM_ALIVE (vom ESP32)
- `CONFIRM_ALIVE <Timestamp>$` --> Wie Echo, schickt Timestamp von ALIVE-Nachricht zurück; Soll das so bald wie möglich machen, auch falls z.B. andere Pakete fehlen

#### STOP (vom ESP32)
- `STOP$` --> Knopf oder ähnliches an ESP32 wurde gedrückt --> Pathfinding beendet, jetzt kann man die micromouse wieder manuell an den Start setzen

#### RESTART (vom ESP32)
- `RESTART$` --> Knopf oder ähnliches nochmal gedrückt --> Nutzer garantiert, dass sich die micromouse wieder am Start befindet

#### BATTERY (vom ESP32)
- `BATTERY <X>$` --> X= pos. Ganzzahl zwischen 0 und 100 --> Batterie in Prozent

### Bsp

(`<` bedeutet zu dem ESP32, `>` bedeutet zu dem PI)

#### Simple Bewegungen, nacheinander

```msgs
< MOVE #0 2$
> CMD_FINISHED #0$
< TURN #1 1$
> CMD_FINISHED #1$
< MOVE #2 1$
> CMD_FINISHED #2$
< TURN #3 -1$
> CMD_FINISHED #3$
```

#### Simple Bewegungen, batched

```msgs
< MOVE #0 2$
< TURN #1 1$
< MOVE #2 1$
< TURN #3 -1$
> CMD_FINISHED #0$
> CMD_FINISHED #1$
> CMD_FINISHED #2$
> CMD_FINISHED #3$
```

#### Simple Bewegungen mit Measurements

```msgs
< MOVE #0 4 MEASURE 1_L_CONTINUE 2_R_CONTINUE$
> MEASUREMENT #0 1_L 2$
> MEASUREMENT #0 2_R 3 SENSORLIMIT$
> CMD_FINISHED #0$
```
