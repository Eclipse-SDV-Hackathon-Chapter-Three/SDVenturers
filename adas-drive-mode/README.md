# Process Mood Signals

A Rust application that subscribes to MQTT topics for driver mood and heartbeat data, processing them asynchronously and analyzing the combined data every second.

## Features

- Subscribes to two MQTT topics:
  - `driver/mood` - Receives driver mood data
  - `driver/heartbeat` - Receives driver heartbeat signals
- Collects data asynchronously as it arrives in vectors
- Performs comprehensive analysis every 10 seconds on collected data
- Risk assessment based on mood and heart rate correlation
- Alert system for high-risk situations

## Data Structures

### Driver Mood Data
```json
{
  "mood": "happy"
}
```

Mood values can be: `"happy"`, `"neutral"`, or `"sad"`

### Heartbeat Signal Data
```json
{
  "heart_rate": 75
}
```

Heart rate is a simple integer representing beats per minute.

## Prerequisites

1. **MQTT Broker**: You need an MQTT broker running. For local testing, you can use Mosquitto:
   ```bash
   # Install Mosquitto (Ubuntu/Debian)
   sudo apt-get install mosquitto mosquitto-clients
   
   # Start the broker
   sudo systemctl start mosquitto
   sudo systemctl enable mosquitto
   ```

2. **Rust**: Make sure you have Rust installed:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

## Building and Running

1. **Build the application**:
   ```bash
   cargo build --release
   ```

2. **Run the application**:
   ```bash
   # Set log level for detailed output
   RUST_LOG=info cargo run
   ```

## Testing

The project includes a Python test publisher that sends sample data to test the application.

1. **Install Python dependencies**:
   ```bash
   pip install paho-mqtt
   ```

2. **Make the test script executable**:
   ```bash
   chmod +x test_publisher.py
   ```

3. **Run the test publisher** (in a separate terminal):
   ```bash
   python3 test_publisher.py
   ```

4. **Monitor MQTT messages** (optional, for debugging):
   ```bash
   # Subscribe to all topics
   mosquitto_sub -h localhost -t "driver/#" -v
   
   # Subscribe to specific topics
   mosquitto_sub -h localhost -t "driver/mood" -v
   mosquitto_sub -h localhost -t "driver/heartbeat" -v
   ```

## Configuration

The application currently connects to `localhost:1883` by default. To modify the MQTT broker settings, update the `MqttOptions` in `src/main.rs`:

```rust
let mut mqttoptions = MqttOptions::new("mood-heartbeat-processor", "your-broker-host", 1883);
```

## Risk Assessment

The application performs comprehensive risk assessment based on:

### Mood Analysis
- **Pattern Recognition**: Identifies sad mood frequency and concerning patterns
- **Threshold Detection**: Alerts when sad moods exceed 50% of readings

### Emotion Analysis (Camera Priority)
- **Pattern Recognition**: Analyzes mood frequency to determine dominant emotion
- **Emotion Output**: Returns "sad", "happy", or "neutral" based on majority patterns
- **Camera Priority**: Emotion detection takes precedence in final decision making

### Heartbeat Analysis 
- **Simple Trend Detection**: Compares first and last readings to determine direction
- **Classification Output**: Returns "high" or "nominal" based on trend and elevation
- **Falling Trend Logic**: Always classifies as "nominal" if heart rate is declining
- **Rising/Stable Logic**: Classifies as "high" if >50% readings are elevated (>100 bpm)

### Combined Analysis (Camera-Prioritized)
- **Emotion-First Decision**: Camera emotion data takes priority over heartbeat
- **Sad Emotion**: Always triggers alerts regardless of heartbeat status
- **Happy Emotion**: Positive override - high heartbeat assumed to be physical activity
- **Neutral Emotion**: Secondary consideration given to heartbeat data

Decision Matrix (Camera Priority):
- **CRITICAL**: Sad emotion + High heartbeat
- **HIGH**: Sad emotion + Nominal heartbeat (emotion priority)  
- **MEDIUM**: Neutral emotion + High heartbeat, or Happy emotion + High heartbeat
- **NORMAL**: Happy/Neutral emotion + Nominal heartbeat, or any Falling heart rate trend

### Key Features
- **Camera Priority**: Emotional state from camera overrides physiological data
- **Contextual Intelligence**: Happy emotion + high heartbeat = likely exercise, not distress
- **Simplified Trend Logic**: Falling heart rate trend always results in "nominal" classification
- **Clear Decision Path**: Emotion → Heartbeat → Final Decision with explicit priority reasoning

## Logging

The application uses structured logging. Set the `RUST_LOG` environment variable to control log levels:

```bash
# Show all logs
RUST_LOG=debug cargo run

# Show info and above
RUST_LOG=info cargo run

# Show warnings and errors only
RUST_LOG=warn cargo run
```

## Architecture

- **Async MQTT Client**: Handles incoming messages from both topics
- **Data Collection**: Thread-safe storage collecting data in vectors using `Arc<Mutex<CollectedData>>`
- **Batch Processor**: Runs every 10 seconds to analyze collected data patterns
- **Pattern Analysis Engine**: Evaluates trends, statistics, and correlations in collected data
- **Multi-level Risk Assessment**: Individual pattern analysis plus combined correlation analysis

## Future Enhancements

- Database persistence for historical data
- RESTful API for external integrations
- Dashboard for real-time monitoring
- Machine learning models for better risk prediction
- Configurable alert thresholds
- Integration with external notification systems