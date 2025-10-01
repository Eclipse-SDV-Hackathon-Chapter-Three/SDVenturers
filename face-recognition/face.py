import cv2
import json
import time
import paho.mqtt.client as mqtt
from deepface import DeepFace

BROKER = "localhost"
PORT = 1883
TOPIC = "driver/mood"

client = mqtt.Client()
client.connect(BROKER, PORT, 60)
client.loop_start()

cap = cv2.VideoCapture(0)

try:
    while True:
        ret, frame = cap.read()
        if not ret:
            break

        try:
            results = DeepFace.analyze(
                img_path=frame,
                actions=['emotion'],
                enforce_detection=False
            )

            results = results if isinstance(results, list) else [results]

            for face in results:
                x, y, w, h = face['region']['x'], face['region']['y'], face['region']['w'], face['region']['h']
                dominant_emotion = face['dominant_emotion']

                # Draw bounding box + label with only mood
                cv2.rectangle(frame, (x, y), (x + w, y + h), (0, 255, 0), 2)
                cv2.putText(frame, dominant_emotion, (x, y - 10),
                            cv2.FONT_HERSHEY_SIMPLEX, 0.9, (0, 255, 0), 2)

                # Print only mood
                print(f"mood: {dominant_emotion}")

                # Publish only mood as JSON
                payload = {
                    "mood": dominant_emotion
                }
                client.publish(TOPIC, json.dumps(payload))
                time.sleep(0.50)

        except Exception as e:
            print("No face detected:", e)

        cv2.imshow("DeepFace Emotion Recognition", frame)
        if cv2.waitKey(1) & 0xFF == ord('q'):
            break

finally:
    cap.release()
    cv2.destroyAllWindows()
    client.loop_stop()
    client.disconnect()
