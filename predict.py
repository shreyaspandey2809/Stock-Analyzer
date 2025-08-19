import json
import numpy as np
import pandas as pd
import os

SHORT_WINDOW = 5
LONG_WINDOW = 15 
LOOKBACK_DAYS = 20 

if not os.path.exists("stock_data.json"):
    print("❌ Error: stock_data.json not found!")
    exit(1)

with open("stock_data.json", "r") as f:
    try:
        data = json.load(f)
    except json.JSONDecodeError:
        print("❌ Error: Could not read stock_data.json (invalid JSON).")
        exit(1)

if not data or not isinstance(data, list):
    print("❌ Error: Invalid data format in stock_data.json.")
    exit(1)

df = pd.DataFrame(data, columns=["date", "price"])

df = df.tail(LOOKBACK_DAYS).reset_index(drop=True)

df["EMA_short"] = df["price"].ewm(span=SHORT_WINDOW, adjust=False).mean()
df["EMA_long"] = df["price"].ewm(span=LONG_WINDOW, adjust=False).mean()

if df["EMA_short"].iloc[-1] > df["EMA_long"].iloc[-1]:
    trend = "Uptrend 📈"
    recommendation = "BUY ✅"
elif df["EMA_short"].iloc[-1] < df["EMA_long"].iloc[-1]:
    trend = "Downtrend 📉"
    recommendation = "SELL ❌"
else:
    trend = "Sideways ➡️"
    recommendation = "HOLD ⚠️"

dates = np.arange(len(df))
prices = df["price"].values
coef = np.polyfit(dates, prices, 1) 
predicted_price = coef[0] * (len(df)) + coef[1]

ai_output = {
    "trend": trend,
    "predicted_price": round(predicted_price, 2),
    "recommendation": recommendation
}

with open("ai_output.json", "w") as f:
    json.dump(ai_output, f, indent=4)

print("Short-term AI prediction complete.")