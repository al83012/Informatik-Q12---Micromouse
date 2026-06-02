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

ssh arne_lender@micromouse-pi

netsh wlan disconnect
# Reconnect to previous WiFi (only if it existed)
if ($currentSSID -and $currentSSID -ne $ssid) {
    Write-Host "Reconnecting to: $currentSSID"
    netsh wlan connect name="$currentSSID"
}
