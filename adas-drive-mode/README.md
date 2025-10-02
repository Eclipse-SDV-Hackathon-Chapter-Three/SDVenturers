# ADAS Drive Mode Controller

A Rust application that monitors driver mood and heart rate to automatically adjust vehicle drive modes for safety.

## Features

- Monitors driver emotional state and heart rate via MQTT
- Analyzes data every 10 seconds to determine driver state
- Automatically switches between "normal" and "restricted" drive modes
- Prioritizes emotional state over heart rate for safety decisions
- Publishes drive mode decisions via uProtocol
- Ready for containerized deployment with Podman

## Data Format

**Input - Driver Mood:**
```json
{"mood": "happy"}
```
Supported: `happy`, `neutral`, `sad`, `angry`

**Input - Heart Rate:**
```json
{"heart_rate": 75}
```

**Output - Drive Mode:**
```json
{"mode": "normal", "reason": "Emotion: happy, Heartbeat: nominal"}
```
Modes: `normal` or `restricted`

## Prerequisites

- **Podman**: For running containers
  ```bash
  sudo apt-get install podman
  ```

- **Rust**: For building from source (optional)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

## Quick Start

**Using Containers (Recommended):**
```bash
# Build image
podman build -t adas-drive-mode-app:0.1 .

# Deploy with Ankaios
ank apply adas-drive-mode.yaml
```

**Local Development:**
```bash
# Build and run
cargo build --release
RUST_LOG=info cargo run
```
## Configuration

**Environment Variables:**
- `MQTT_BROKER_HOST`: Broker hostname (default: localhost)
- `MQTT_BROKER_PORT`: Broker port (default: 1883)
- `RUST_LOG`: Log level (info, debug, warn, error)

**MQTT Topics:**
- Subscribes to: `driver/mood`, `driver/heart_rate`
- Publishes via uProtocol to: `drive.mode/2/4//0x8001`

## How It Works

**Decision Logic:**
- **Sad/Angry emotion** → Always `restricted` mode (safety first)
- **Happy emotion** → Always `normal` mode (even with high heart rate)
- **Neutral emotion** → Depends on heart rate
- **High heart rate** → `restricted` mode (unless happy emotion)
- **Falling heart rate** → Always considered `normal`

**Analysis:**
- Processes data every 10 seconds
- Emotion takes priority over heart rate
- Heart rate >100 bpm considered "high"
- Trend analysis: rising, falling, or stable

## Development

**View logs:**
```bash
RUST_LOG=info cargo run  # Basic info
RUST_LOG=debug cargo run # Detailed logs
```

**Tech stack:**
- Rust with Tokio async runtime
- MQTT for data input, uProtocol for output
- 10-second batch processing intervals
- Containerized with Podman and Ankaios