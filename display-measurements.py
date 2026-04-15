import sys
import pandas as pd
import matplotlib.pyplot as plt

file = sys.argv[1]

# Define column names (in the exact order of your CSV)
columns = [
    "date",
    "time",
    "weight_0",
    "weight_1",
    # "weight_total",
    "weight_min",
    "weight_max",
    "weight_desired"
]

# Load CSV without header
df = pd.read_csv(file, header=None, names=columns)

# Combine date + time
df["datetime"] = pd.to_datetime(
    df["date"].astype(str) + " " + df["time"].astype(str),
    dayfirst=True,
    format="mixed"
)

# Sort by time
df = df.sort_values("datetime")

# Plot all weight columns
for col in columns[2:]:  # skip date, time
    df[col] = pd.to_numeric(df[col], errors="coerce")
    plt.plot(df["datetime"], df[col], label=col)

# Labels
plt.xlabel("Time")
plt.ylabel("Weight")
plt.title("Weight Data")
plt.legend()

# Improve readability
plt.xticks(rotation=45)
plt.tight_layout()

plt.show()