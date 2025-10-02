import tkinter as tk
from PIL import Image, ImageTk
import time
import os

# Pozisyonlar
x_center = 335
y_bottom = 110
checkbox_x = 600   # sağ üst köşe
checkbox_y = 20

class ClusterUI:
    def __init__(self, root):
        self.root = root
        self.root.title("Cluster Display")
        self.root.geometry("670x300")
        self.root.configure(bg="black")

        # Üstte kontrol alanı
        control_frame = tk.Frame(root, bg="black")
        control_frame.pack(side="top", fill="x")

        self.entry = tk.Entry(control_frame)
        self.entry.pack(side="left", padx=5, pady=5)

        self.btn = tk.Button(control_frame, text="Test Mode", command=self.manual_test)
        self.btn.pack(side="left", padx=5, pady=5)

        # Canvas
        self.canvas = tk.Canvas(root, width=670, height=251, bg="black", highlightthickness=0)
        self.canvas.pack()

        # Script’in bulunduğu dizini bul
        BASE_DIR = os.path.dirname(os.path.abspath(__file__))
        img_path = os.path.join(BASE_DIR, "..", "images", "cluster.png")

        # Arka plan görseli
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
        """Mode'a göre checkbox çiz (sabit kalacak)"""
        if self.checkbox_item:
            self.canvas.delete(self.checkbox_item)

        color = "green" if mode == "normal" else "red"
        self.checkbox_item = self.canvas.create_rectangle(
            checkbox_x - 15, checkbox_y - 15,
            checkbox_x + 15, checkbox_y + 15,
            fill=color, outline="white"
        )

    def start_blink_text(self, text="⚠ Restricted Mode!"):
        """Yalnızca yazıyı blink yap"""
        if not self.blinking:
            self.blinking = True
            self.start_time = time.time()
            self._blink_text(text)

    def _blink_text(self, text):
        elapsed = time.time() - self.start_time
        if elapsed > 30:  # 30 seconds blink
            self.blinking = False
            if self.popup_text:
                self.canvas.delete(self.popup_text)
            return

        # Renk toggle
        color = "red" if int(elapsed * 2) % 2 == 0 else "black"

        # Öncekini sil
        if self.popup_text:
            self.canvas.delete(self.popup_text)

        # Yeni text çiz
        self.popup_text = self.canvas.create_text(
            x_center, y_bottom,
            text=text,
            fill=color, font=("Arial", 14, "bold")
        )

        # 500 ms sonra tekrar çağır
        self.root.after(500, lambda: self._blink_text(text))

    def manual_test(self):
        value = self.entry.get().strip().lower()
        if value == "normal":
            self.draw_checkbox("normal")
        elif value == "restricted":
            self.draw_checkbox("restricted")
            self.start_blink_text("⚠ Driver Anger Detected!")
        else:
            print("Geçersiz değer:", value)


if __name__ == "__main__":
    root = tk.Tk()
    app = ClusterUI(root)
    root.mainloop()
