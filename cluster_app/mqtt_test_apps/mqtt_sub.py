import paho.mqtt.client as mqtt

broker = "192.168.24.254"
port = 1883
topic = "drive.mode/2/0/4/8001"

def on_message(client, userdata, msg):
    print(f"Message: {msg.payload.decode()}")

client = mqtt.Client()
client.connect(broker, port, 60)

client.subscribe(topic)
client.on_message = on_message

client.loop_forever()
