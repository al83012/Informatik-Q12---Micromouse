
# Raspberry Pi Headless + AP + Rust Cross-Compile Setup Guide

* Configure Raspberry Pi as a WiFi Access Point
* Cross-compile Rust code from Windows (via WSL)
* Transfer binaries over SSH
* Run automatically on boot (console-only)

---

# 1. Base Raspberry Pi Setup

## Enable SSH

```bash
sudo raspi-config
```

→ Interface Options → SSH → Enable

---

## Boot to Console (no GUI)

```bash
sudo raspi-config
```

→ System Options → Boot / Auto Login → Console (no GUI)

Or manually:

```bash
sudo systemctl set-default multi-user.target
```

---

# 2. Configure Raspberry Pi as Access Point

## Install required packages

```bash
sudo apt update
sudo apt install hostapd dnsmasq -y
```

Stop services for now:

```bash
sudo systemctl stop hostapd
sudo systemctl stop dnsmasq
```

---

## Configure static IP

Edit:

```bash
sudo nano /etc/dhcpcd.conf
```

Add:

```text
interface wlan0
static ip_address=192.168.4.1/24
nohook wpa_supplicant
```

---

## Configure DHCP server

Backup original config:

```bash
sudo mv /etc/dnsmasq.conf /etc/dnsmasq.conf.orig
```

Create new:

```bash
sudo nano /etc/dnsmasq.conf
```

```text
interface=wlan0
dhcp-range=192.168.4.2,192.168.4.20,255.255.255.0,24h
```

---

## Configure Access Point

```bash
sudo nano /etc/hostapd/hostapd.conf
```

```text
interface=wlan0
driver=nl80211
ssid=MicromousePi
hw_mode=g
channel=7
wmm_enabled=0
macaddr_acl=0
auth_algs=1
ignore_broadcast_ssid=0
wpa=2
wpa_passphrase=yourpassword
wpa_key_mgmt=WPA-PSK
rsn_pairwise=CCMP
```

---

## Link config

```bash
sudo nano /etc/default/hostapd
```

Set:

```text
DAEMON_CONF="/etc/hostapd/hostapd.conf"
```

---

## Enable services

```bash
sudo systemctl unmask hostapd
sudo systemctl enable hostapd
sudo systemctl enable dnsmasq
```

Reboot:

```bash
sudo reboot
```

---

## Connect from PC

Connect to:

```
SSID: MicromousePi
IP:   192.168.4.1
```

---

# 3. Rust Cross Compilation (Windows + WSL)

## Install WSL

```powershell
wsl --install
```

---

## Inside WSL

Install dependencies:

```bash
sudo apt update
sudo apt install build-essential curl git docker.io -y
```

---

## Install Rust

```bash
curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env
```

---

## Install cross

```bash
cargo install cross
```

---

## Build for Raspberry Pi

### For 64-bit OS:

```bash
cross build --release --target aarch64-unknown-linux-gnu
```

### (Optional) Static binary:
(PREFERRED --> fewer points of error)

```bash
cross build --release --target aarch64-unknown-linux-musl
```

---

# 4. Transfer Binary via SSH

From Windows (PowerShell):

```bash
scp target/aarch64-unknown-linux-gnu/release/backend-micromouse pi@192.168.4.1:/home/pi/
```

---

## Run manually

```bash
ssh pi@192.168.4.1
chmod +x backend-micromouse
./backend-micromouse
```

---

# 5. Automate WiFi Switching + Deploy

## PowerShell Script Example

```powershell
$piSSID = "MicromousePi"

# Save current WiFi
$currentSSID = (netsh wlan show interfaces |
    Select-String '^\s*SSID\s*:\s*(.+)$').Matches.Groups[1].Value.Trim()

# Connect to Pi
netsh wlan connect name=$piSSID
Start-Sleep -Seconds 5

# Upload + run
scp backend-micromouse pi@192.168.4.1:/home/pi/
ssh pi@192.168.4.1 "chmod +x backend-micromouse && ./backend-micromouse"

# Reconnect previous WiFi
if ($currentSSID -and $currentSSID -ne $piSSID) {
    netsh wlan connect name="$currentSSID"
}
```

---

# 6. Auto-start Program on Boot (Console)

Edit:

```bash
nano ~/.bash_profile
```

Add:

```bash
if [ "$(tty)" = "/dev/tty1" ] && [ -z "$SSH_CONNECTION" ]; then
    while true; do
        /home/pi/backend-micromouse
        echo "Program crashed. Restarting..."
        sleep 2
    done
fi
```

---

## Result

On boot:

* Pi shows console on HDMI
* Program runs automatically
* Output visible directly
* Restarts if it crashes

---

# 7. (Optional) Control Onboard LEDs

Disable trigger:

```bash
echo none | sudo tee /sys/class/leds/led0/trigger
```

Control:

```bash
echo 1 | sudo tee /sys/class/leds/led0/brightness
echo 0 | sudo tee /sys/class/leds/led0/brightness
```

---

# 8. Debugging Tips

## Check architecture

```bash
uname -m
file backend-micromouse
```

---

## Check logs

```bash
dmesg
```

---

## Test binary

```bash
./backend-micromouse
```

---

# Final Setup Summary

* Pi acts as WiFi Access Point
* PC connects directly to Pi
* Rust builds via WSL + cross
* Binary sent via SCP
* Program auto-runs on boot
* Output visible on HDMI console

---

# Optional Improvements

* Use static builds (`musl`) for portability
* Add LED status indicators
* Implement remote logging
* Add watchdog / health checks

---
