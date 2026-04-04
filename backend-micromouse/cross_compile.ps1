
# First: need to compile in WSL using CROSS
wsl -d Ubuntu bash -lc "cross build --release --target aarch64-unknown-linux-musl"

$ssid = "micromouse_pi_hotspot"

# Get currently connected WiFi SSID
$currentSSID = (netsh wlan show interfaces |
    Select-String '^\s*SSID\s*:\s*(.+)$').Matches.Groups[1].Value.Trim()

Write-Host "Current WiFi: $currentSSID"

# Connect to Raspberry Pi hotspot
netsh wlan disconnect
Write-Host "Disconnected"

netsh wlan connect name=$ssid

Write-Host "New connection"

# Wait for connection
Start-Sleep -Seconds 5

# Deploy + run
scp "C://Users/arnel/Desktop/Other/Schule/Informatik - 12/Projekt - Q12_2/Informatik-Q12---Micromouse/backend-micromouse/target/aarch64-unknown-linux-musl/release/backend-micromouse" arne_lender@micromouse-pi:/home/arne_lender/
ssh arne_lender@micromouse-pi "chmod +x ./backend-micromouse && ./backend-micromouse"


netsh wlan disconnect
# Reconnect to previous WiFi (only if it existed)
if ($currentSSID -and $currentSSID -ne $ssid) {
    Write-Host "Reconnecting to: $currentSSID"
    netsh wlan connect name="$currentSSID"
}
