import paho.mqtt.client as mqtt

broker = "localhost"   # broker senin bilgisayarında
port = 1883
topic = "drive.mode/2/0/4/8001"

# MQTT client oluştur
client = mqtt.Client()
client.connect(broker, port, 60)

# Mesaj gönder
client.publish(topic, "anger")   # subscriber tarafında popup çıkacak
client.disconnect()


