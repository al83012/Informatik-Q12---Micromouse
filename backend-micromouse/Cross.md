# Cross-compilation für den RaspberryPi

Für cross-compilation für den Raspberry Pi sind einige Voraussetzungen erforderlich (Für Windows):

Auf Windows:

- Podman installieren
- Docker Desktop installieren
- WSL Ubuntu installieren (`wsl --install`)
- Docker Desktop starten

In WSL:

- `rustup update
cargo install cross --force`
- `cross build --target aarch64-unknown-linux-musl --verbose`
- Falls der Zugriff zu Docker nicht funktioniert: zu Gruppe hinzufügen
  - `sudo usermod -aG docker $USER`
  - Neustart

Der Raspberry PI ist gerade unter "arne_lender@micromouse-pi" zu finden; Verbindung kann via `ssh arne_lender@micrmouse-pi` aufgebaut werden
