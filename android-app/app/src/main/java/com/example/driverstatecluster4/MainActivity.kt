package com.example.driverstatecluster4

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.DialogProperties
import com.example.driverstatecluster4.ui.theme.DriverStateCluster4Theme
import org.eclipse.paho.client.mqttv3.*
import android.widget.Toast
import kotlinx.coroutines.delay

class MainActivity : ComponentActivity() {
    private lateinit var mqttClient: MqttClient
    private val serverUri = "tcp://192.168.24.254:1883"
    private val topic = "drive.mode/2/0/4/8001"
    private val clientId = "DriverStateClient-${System.currentTimeMillis()}"

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            DriverStateCluster4Theme {
                var showDialog by remember { mutableStateOf(false) }
                var dialogMessage by remember { mutableStateOf("") }
                var currentEmotion by remember { mutableStateOf("") }
                var currentDriveMode by remember { mutableStateOf("Normal") }
                val context = LocalContext.current

                // Temporary manual trigger for testing the popup
                LaunchedEffect(Unit) {
                    // Simulate mood detection for testing
                    currentEmotion = "sad"
                    currentDriveMode = "Restricted"
                    dialogMessage = "Hello Driver, yoe seem sad. Mode switched to: $currentDriveMode."
                    showDialog = true
                    delay(5000L) // Delay within coroutine scope
                    showDialog = false
                }

                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    Greeting(
                        name = "driver",
                        emotion = currentEmotion,
                        driveMode = currentDriveMode,
                        modifier = Modifier.padding(innerPadding)
                    )
                }

                if (showDialog) {
                    AlertDialog(
                        onDismissRequest = { showDialog = false },
                        title = { Text("Drive Mode Change Alert") },
                        text = { Text(dialogMessage) },
                        confirmButton = {
                            Button(onClick = { showDialog = false }) {
                                Text("Acknowledge")
                            }
                        },
                        properties = DialogProperties(usePlatformDefaultWidth = false),
                        modifier = Modifier
                            .fillMaxWidth(0.6f) // 60% width leaves 20% margins on each side
                            .padding(horizontal = 20.dp) // Additional padding for consistency
                    )
                }
            }
        }
    }


    override fun onDestroy() {
        super.onDestroy()
    }
}

@Composable
fun Greeting(name: String, emotion: String, driveMode: String, modifier: Modifier = Modifier) {
    val message = if (emotion in listOf("sad", "angry")) {
        "Hello $name, you seem $emotion; drive mode is $driveMode"
    } else {
        "Hello $name; drive mode is $driveMode"
    }
    Text(
        text = message,
        modifier = modifier
    )
}

@Preview(showBackground = true)
@Composable
fun GreetingPreview() {
    DriverStateCluster4Theme {
        Greeting(name = "driver", emotion = "sad", driveMode = "Restricted")
    }
}