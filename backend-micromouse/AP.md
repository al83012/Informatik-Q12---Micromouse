RaspberryPi -> `arne_lender@micromouse-pi`

Als AccessPoint konfigurieren:
[Guide](https://www.raspberrypi.com/tutorials/host-a-hotel-wifi-hotspot/)

- über `ssh arne_lender@micromouse-pi` mit RP über Router verbinden
- `nmcli device` --> Auflisten der Verbindungen
- `eth0` muss verbunden sein, weil sonst beim Rekonfigurieren die Verbindung über ssh abbricht
- `sudo nmcli device wifi hotspot ssid <hotspot name> password <hotspot password> ifname wlan0`
- Hier: `micromouse-pi-hotspot` und `12345678` (Weil Sicherheit nicht das höchste Ziel ist)
- `ssh arne_lender@micromouse-pi.local`
- `nmcli connection` --> Sollte Hotspot zeigen
- `sudo nmcli connection modify <hotspot UUID> connection.autoconnect yes connection.autoconnect-priority 100`


Oder anderer [Guide](https://raspberrypi-guide.github.io/networking/create-wireless-access-point)

- Evtl. falls dhcpcd nicht gefunden wird --> via `sudo apt-get install dhcpcd` clean install

