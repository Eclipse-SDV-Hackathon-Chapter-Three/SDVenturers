
"""
    @file_name: cluster_ui.py
    @author: Batuhan Arslan
    @date:   01/10/2025
    @brief:  Simple cluster UI giving outputs to the driver.
    @attentions: 
    

"""
import tkinter as tk
from PIL import Image, ImageTk
import paho.mqtt.client as mqtt
import json
import time
import os

broker = "192.168.24.254"
port = 1883
topic = "drive.mode/2/0/4/8001"

# Pozisyonlar
x_center = 335
y_bottom = 110
checkbox_x = 600   # sağ üst köşe
checkbox_y = 40

class ClusterUI:
    def __init__(self, root):
        self.root = root
        self.root.title("Cluster Display")
        self.root.geometry("670x251")
        self.root.configure(bg="black")

        # Canvas
        self.canvas = tk.Canvas(root, width=670, height=251, bg="black", highlightthickness=0)
        self.canvas.pack()

        # Script’in bulunduğu dizini bul
        BASE_DIR = os.path.dirname(os.path.abspath(__file__))
        img_path = os.path.join(BASE_DIR, "..", "images", "cluster.png")

        self.bg_image = Image.open(img_path).resize((670, 251))
        self.bg_photo = ImageTk.PhotoImage(self.bg_image)
        self.canvas.create_image(0, 0, image=self.bg_photo, anchor="nw")

        # Objeler
        self.checkbox_item = None
        self.popup_text = None

        # Blink kontrolü
        self.blinking = False
        self.start_time = None

    def draw_checkbox(self, mode):
        """Mode'a göre checkbox çiz"""
        if self.checkbox_item:
            self.canvas.delete(self.checkbox_item)

        color = "green" if mode == "normal" else "red"
        self.checkbox_item = self.canvas.create_rectangle(
            checkbox_x - 15, checkbox_y - 15,
            checkbox_x + 15, checkbox_y + 15,
            fill=color, outline="white"
        )

    def start_blink_text(self, text="⚠ Restricted Mode!"):
        #"blink only the text on the screen"
        if not self.blinking:
            self.blinking = True
            self.start_time = time.time()
            self._blink_text(text)

    def _blink_text(self, text):
        elapsed = time.time() - self.start_time
        if elapsed > 30:  #30 seconds blink for driver interaction
            self.blinking = False
            if self.popup_text:
                self.canvas.delete(self.popup_text)
            return

        # color toggle
        color = "red" if int(elapsed * 2) % 2 == 0 else "black"

        # delete previous one
        if self.popup_text:
            self.canvas.delete(self.popup_text)

        # Yeni text çiz
        self.popup_text = self.canvas.create_text(
            x_center, y_bottom,
            text=text,
            fill=color, font=("Arial", 14, "bold")
        )

        #blink 500ms every cycle
        self.root.after(500, lambda: self._blink_text(text))


# MQTT classback side
def on_connect(client, userdata, flags, rc):
    print("MQTT connected with result code " + str(rc))
    client.subscribe(topic)

def on_message(client, userdata, msg):
    try:
        payload = msg.payload.decode()
        print(f"Gelen mesaj: {payload}")
        data = json.loads(payload)

        mode_value = data.get("mode", "unknown")

        if mode_value == "normal":
            app.root.after(0, lambda: app.draw_checkbox("normal"))

        elif mode_value == "restricted":
            app.root.after(0, lambda: app.draw_checkbox("restricted"))
            app.root.after(0, lambda: app.start_blink_text("⚠ Driver Anger Detected!"))

    except Exception as e:
        print("JSON parse error:", e)

# Main part functions called.
if __name__ == "__main__":
    root = tk.Tk()
    app = ClusterUI(root)

    client = mqtt.Client()
    client.on_connect = on_connect
    client.on_message = on_message
    client.connect(broker, port, 60)

    client.loop_start()
    root.mainloop()
