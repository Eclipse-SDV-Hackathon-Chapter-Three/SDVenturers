# Team 4: SDVenturers: Mood Detection and Action

Repository structure:

- adas-drive-mode:
    - Rust app for mood detection decision logic
- face recognition:
    - Python app using deepface for mood detection via camera
- python-dashboard:
    - Python dashboard app for visualizing mood detection and car behavior
- sdv_lab:
    - Fork of sdv_lab repo with:
      - sdv_lab/android_treadx/threadx/threadx-app/cross/sdventurers_app:
        - Rust thread app for AZ3166 mcu as a mock radar sensor detecting heart rate
      - Android digital cluster dashboard app for visualizing mood detection and car behavior

## Team Members

- **Batuhan Arslan**  
  GitHub: [engbatuhanarslan](https://github.com/engbatuhanarslan)
  Role: Captain
- **Tessa Talsma**  
  GitHub: [ttalsma](https://github.com/ttalsma)
  Role: Navigator
- **Adarsh Rastogi**  
  GitHub: [adarshrastogi67](https://github.com/adarshrastogi67)
  Role: Challenger
- **Aniket Barve**  
  GitHub: [Aniket-3005](https://github.com/Aniket-3005)
  Role: Strategist
- **Pascal Kneuper**  
  GitHub: [k411e](https://github.com/k411e)
  Role: Explorer

---

## Challenge

**SDV Lab**

### Core Idea: Mood Detection and Action

- Driver mood detection (e.g., via bio signals such as heart rate, eye behavior)
- Limit or adjust car/ECU behavior accordingly (e.g., limit maximum acceleration)

---

## Development Process

1. **Figure out mood detection:** Facial recognition, AI, etc.
2. **Data collection:**  
   - Which sensors?  
   - Which signals?  
   - Which algorithms?
3. **Scope of car/ECU behavior changes and implementation**
4. **Testing:**  
   - Via three simple scenarios (e.g., 3 face emotions)
5. **Communication/Sharing:**  
   - Via Slack/whiteboard  
   - Decisions made via voting
