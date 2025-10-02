use anyhow::Result;
use log::{error, info, warn};
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::interval;
use up_rust::{UMessageBuilder, UPayloadFormat, UTransport, UUri};
use up_transport_mqtt5::{Mqtt5Transport, Mqtt5TransportOptions, MqttClientOptions};

// subscribe
const TOPIC_DRIVER_MOOD: &str = "driver/mood";
const TOPIC_DRIVER_HEARTRATE: &str = "driver/heart_rate";

// publish
// const TOPIC_DRIVE_MODE: &str = "vehicle/drive_mode";
const X_AUTH: &str = "drive.mode";
const UEID: u32 = 2;
const VERSION: u8 = 4;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DriverMood {
    mood: String, // "neutral", "happy", "sad", "angry"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HeartbeatSignal {
    heart_rate: u32, // Simple integer value
}

#[derive(Debug, Clone)]
struct CollectedData {
    mood_data: Vec<DriverMood>,
    heartbeat_data: Vec<HeartbeatSignal>,
    last_processed: std::time::Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DriveMode {
    mode: String, // e.g., "normal", "restricted"
    reason: String, // Explanation for the mode
}

type SharedData = Arc<Mutex<CollectedData>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();
    let mqtt_broker_host = std::env::var("MQTT_BROKER_HOST").unwrap_or_else(|_| "localhost".to_string());
    let mqtt_broker_port = std::env::var("MQTT_BROKER_PORT").unwrap_or_else(|_| "1883".to_string());
    
    // Shared data store for collected information
    let shared_data: SharedData = Arc::new(Mutex::new(CollectedData {
        mood_data: Vec::new(),
        heartbeat_data: Vec::new(),
        last_processed: std::time::Instant::now(),
    }));

    // uprotocol configuration
    // --- MQTT5 Transport Specific Stuff ---
    let mqtt_client_options = MqttClientOptions {
        broker_uri: format!("{mqtt_broker_host}:{mqtt_broker_port}"),
        // broker_uri: "192.168.24.247:1883".to_string(),
        ..Default::default()
    };

    let mqtt_transport_options = Mqtt5TransportOptions {
        mqtt_client_options,
        ..Default::default()
    };

    info!("Creating uprotocol client");
    let u_protocol_client = Arc::new(Mqtt5Transport::new(mqtt_transport_options, X_AUTH.to_string()).await?);
    // Connect to broker
    u_protocol_client.connect().await?;
    // --- End of MQTT5 Transport Specific Stuff ---
    info!("Connected to uProtocol MQTT5 broker");
    
    // MQTT configuration
    let mut mqttoptions = MqttOptions::new("mood-heartbeat-processor", mqtt_broker_host, mqtt_broker_port.parse()?);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    
    // Subscribe to topics
    client.subscribe(TOPIC_DRIVER_MOOD, QoS::AtLeastOnce).await?;
    client.subscribe(TOPIC_DRIVER_HEARTRATE, QoS::AtLeastOnce).await?;
    
    info!("Subscribed to MQTT topics: driver/mood, driver/heartrate");

    
    
    // Clone shared data for the processing task
    let processing_data = shared_data.clone();
    let processing_client = u_protocol_client.clone();
    
    // Spawn processing task that runs every 10 seconds
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            process_collected_data(&processing_data, &processing_client).await;
        }
    });
    
    // Main MQTT event loop
    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(packet)) => {
                handle_incoming_packet(packet, &shared_data).await;
            }
            Ok(Event::Outgoing(_)) => {
                // Handle outgoing events if needed
            }
            Err(e) => {
                error!("MQTT connection error: {}", e);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn handle_incoming_packet(packet: Incoming, shared_data: &SharedData) {
    match packet {
        Incoming::Publish(publish) => {
            let topic = &publish.topic;
            let payload = String::from_utf8_lossy(&publish.payload);
            
            match topic.as_str() {
                TOPIC_DRIVER_MOOD => {
                    match serde_json::from_str::<DriverMood>(&payload) {
                        Ok(mood_data) => {
                            info!("Received mood data: {:?}", mood_data);
                            update_mood_data(shared_data, mood_data).await;
                        }
                        Err(e) => {
                            error!("Failed to parse mood data: {}, payload: {}", e, payload);
                        }
                    }
                }
                TOPIC_DRIVER_HEARTRATE => {
                    match serde_json::from_str::<HeartbeatSignal>(&payload) {
                        Ok(heartbeat_data) => {
                            info!("Received heartrate data: {:?}", heartbeat_data);
                            update_heartbeat_data(shared_data, heartbeat_data).await;
                        }
                        Err(e) => {
                            error!("Failed to parse heartrate data: {}, payload: {}", e, payload);
                        }
                    }
                }
                _ => {
                    warn!("Received message on unknown topic: {}", topic);
                }
            }
        }
        _ => {
            // Handle other incoming packet types if needed
        }
    }
}

async fn update_mood_data(shared_data: &SharedData, mood_data: DriverMood) {
    let mut data = shared_data.lock().unwrap();
    data.mood_data.push(mood_data);
    info!("Collected {} mood data points", data.mood_data.len());
}

async fn update_heartbeat_data(shared_data: &SharedData, heartbeat_data: HeartbeatSignal) {
    let mut data = shared_data.lock().unwrap();
    data.heartbeat_data.push(heartbeat_data);
    info!("Collected {} heartbeat data points", data.heartbeat_data.len());
}

async fn process_collected_data(shared_data: &SharedData, uprotocol_client: &Arc<Mqtt5Transport>) {
    // Extract data from shared state and release the lock quickly
    let (mood_data, heartbeat_data) = {
        let data = shared_data.lock().unwrap();
        (data.mood_data.clone(), data.heartbeat_data.clone())
    };
    
    let mood_count = mood_data.len();
    let heartbeat_count = heartbeat_data.len();
    
    info!("=== Processing Collected Data ===");
    info!("Collected {} mood data points", mood_count);
    info!("Collected {} heartbeat data points", heartbeat_count);
    
    if mood_count == 0 && heartbeat_count == 0 {
        info!("No data collected in this interval");
        return;
    }
    
    // Process mood data and get emotion output
    let emotion_output: Option<String> = if !mood_data.is_empty() {
        Some(analyze_mood_patterns(&mood_data))
    } else {
        Some("unknown".to_string())
    };
    
    // Process heartbeat data and get heartbeat classification
    let heartbeat_output = if !heartbeat_data.is_empty() {
        Some(analyze_heartbeat_patterns(&heartbeat_data))
    } else {
        Some("unknown".to_string())
    };
    
    // Perform combined analysis if both types of data are available
    if let (Some(emotion), Some(heartbeat)) = (&emotion_output, &heartbeat_output) {
        let drive_mode = determine_drive_mode(emotion, heartbeat);

        // Publish drive mode decision via uprotocol
        if let Err(e) = publish_drive_mode(uprotocol_client, &drive_mode).await {
            error!("Failed to publish drive mode: {}", e);
        } else {
            info!("Successfully published drive mode: {:?}", drive_mode);
        }
    }
    
    // Clear the collected data for the next interval
    {
        let mut data = shared_data.lock().unwrap();
        data.mood_data.clear();
        data.heartbeat_data.clear();
        data.last_processed = std::time::Instant::now();
    }
    
    info!("Data processing completed, buffers cleared");
    info!("=================================");
}

fn analyze_mood_patterns(mood_data: &[DriverMood]) -> String {
    info!("--- Mood Pattern Analysis ---");
    
    let mut mood_counts = HashMap::new();
    for mood in mood_data {
        *mood_counts.entry(mood.mood.clone()).or_insert(0) += 1;
    }
    
    for (mood, count) in &mood_counts {
        let percentage = (*count as f32 / mood_data.len() as f32) * 100.0;
        info!("{}: {} occurrences ({:.1}%)", mood, count, percentage);
    }
    
    // Determine overall emotion output
    let sad_count = mood_counts.get("sad").unwrap_or(&0);
    let happy_count = mood_counts.get("happy").unwrap_or(&0);
    let neutral_count = mood_counts.get("neutral").unwrap_or(&0);
    let angry_count = mood_counts.get("angry").unwrap_or(&0);
    
    let sad_percentage = (*sad_count as f32 / mood_data.len() as f32) * 100.0;
    let happy_percentage = (*happy_count as f32 / mood_data.len() as f32) * 100.0;
    let neutral_percentage = (*neutral_count as f32 / mood_data.len() as f32) * 100.0;
    let angry_percentage = (*angry_count as f32 / mood_data.len() as f32) * 100.0;

    let emotion_output = if (sad_percentage + angry_percentage) >= happy_percentage && sad_percentage >= neutral_percentage {
        "sad"
    } else if happy_percentage >= neutral_percentage {
        "happy"
    } else {
        "neutral"
    };
    
    info!("Emotion Output: {}", emotion_output);
    emotion_output.to_string()
}

fn analyze_heartbeat_patterns(heartbeat_data: &[HeartbeatSignal]) -> String {
    info!("--- Heartbeat Pattern Analysis ---");
    
    let heart_rates: Vec<u32> = heartbeat_data.iter().map(|h| h.heart_rate).collect();
    
    if heart_rates.len() < 2 {
        info!("Insufficient data for trend analysis");
        return "nominal".to_string();
    }
    
    if let (Some(&min_hr), Some(&max_hr)) = (heart_rates.iter().min(), heart_rates.iter().max()) {
        let avg_hr = heart_rates.iter().sum::<u32>() as f32 / heart_rates.len() as f32;
        
        info!("Heart Rate Statistics:");
        info!("  Min: {} bpm", min_hr);
        info!("  Max: {} bpm", max_hr);
        info!("  Average: {:.1} bpm", avg_hr);
        
        // Simple Trend Analysis (first vs last)
        let trend_direction = if heart_rates.last().unwrap() > heart_rates.first().unwrap() {
            "Rising"
        } else if heart_rates.last().unwrap() < heart_rates.first().unwrap() {
            "Falling"
        } else {
            "Stable"
        };
        info!("  Trend: {} ({} -> {} bpm)", trend_direction, heart_rates.first().unwrap(), heart_rates.last().unwrap());
        
        // Determine heartbeat output: "high" or "nominal"
        let elevated_count = heart_rates.iter().filter(|&&hr| hr > 100).count();
        let elevated_percentage = (elevated_count as f32 / heart_rates.len() as f32) * 100.0;
        
        info!("  Elevated HR (>100): {} readings ({:.1}%)", elevated_count, elevated_percentage);
        
        let heartbeat_output = match trend_direction {
            "Falling" => {
                info!("Heartbeat trend is falling - classifying as nominal");
                "nominal"
            }
            "Rising" => {
                if elevated_percentage > 50.0 {
                    warn!("Rising heart rate trend with >50% elevated readings - classifying as high");
                    "high"
                } else {
                    "nominal"
                }
            }
            "Stable" => {
                if elevated_percentage > 50.0 {
                    warn!("Consistently elevated heart rate - classifying as high");
                    "high"
                } else {
                    "nominal"
                }
            }
            _ => "nominal"
        };
        
        info!("Heartbeat Output: {}", heartbeat_output);
        heartbeat_output.to_string()
    } else {
        "nominal".to_string()
    }
}



fn determine_drive_mode(emotion: &str, heartbeat: &str) -> DriveMode {
    info!("--- Combined Pattern Analysis ---");
    info!("Camera Data (Emotion): {}", emotion);
    info!("Heartbeat Data: {}", heartbeat);
    
    // Priority-based decision making with camera data taking precedence
    let drive_mode: DriveMode = match emotion {
        "sad" => {
            // Camera shows sad emotion - this takes priority
            // Verify if heartbeat is also high, else warning message
            match heartbeat {
                "high" => {
                    DriveMode { mode: "restricted".to_string(), reason: "Emotion: sad, Heartbeat: high".to_string() }
                }
                "nominal" => {
                    DriveMode { mode: "restricted".to_string(), reason: "Emotion: sad, Heartbeat: nominal".to_string() }
                }
                _ => {
                    DriveMode { mode: "restricted".to_string(), reason: "Emotion: sad, Heartbeat: unknown".to_string() }
                }
            }

        }
        "happy" => {
            // Camera shows happy emotion - generally positive
            match heartbeat {
                "high" => {
                    DriveMode { mode: "normal".to_string(), reason: "Emotion: happy, Heartbeat: high".to_string() }
                }
                "nominal" => {
                    DriveMode { mode: "normal".to_string(), reason: "Emotion: happy, Heartbeat: nominal".to_string() }
                }
                _ => {
                    DriveMode { mode: "normal".to_string(), reason: "Emotion: happy, Heartbeat: unknown".to_string() }
                }
            }
        }
        "neutral" => {
            // Camera shows neutral emotion - depends more on heartbeat
            match heartbeat {
                "high" => {
                    DriveMode { mode: "normal".to_string(), reason: "Emotion: neutral, Heartbeat: high".to_string() } 
                }
                "nominal" => {
                    DriveMode { mode: "normal".to_string(), reason: "Emotion: neutral, Heartbeat: nominal".to_string() }
                }
                _ => {
                    DriveMode { mode: "normal".to_string(), reason: "Emotion: neutral, Heartbeat: unknown".to_string() }
                }
            }
        }
        _ => {
            // Fallback to heartbeat-only decision
            match heartbeat {
                "high" => {
                    DriveMode { mode: "restricted".to_string(), reason: "Emotion: Unknown, Heartbeat: High".to_string() }
                }
                _ => {
                    DriveMode { mode: "normal".to_string(), reason: "Emotion: Unknown, Heartbeat: ".to_string() + heartbeat }
                }
            }
        }
    };
    
    info!("Drive Mode (Camera Priority): {:?}", drive_mode);
    

    drive_mode
}

async fn publish_drive_mode(uprotocol_client: &Arc<Mqtt5Transport>, drive_mode: &DriveMode) -> Result<()> {
    // Create the publish URI using the defined constants
    let publish_uri = UUri::try_from_parts(X_AUTH, UEID, VERSION, 0x8001)
        .map_err(|e| anyhow::anyhow!("Failed to create publish URI: {}", e))?;
    
    // Serialize the drive mode to JSON
    let payload = serde_json::to_vec(drive_mode)
        .map_err(|e| anyhow::anyhow!("Failed to serialize drive mode: {}", e))?;
    
    // Create the uMessage
    let message = UMessageBuilder::publish(publish_uri)
        .build_with_payload(payload, UPayloadFormat::UPAYLOAD_FORMAT_JSON)?;
    
    // Send the message
    uprotocol_client.send(message).await?;
    
    Ok(())
}
