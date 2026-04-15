import sys
import pandas as pd
import matplotlib.pyplot as plt

file = sys.argv[1]

# Define column names (in the exact order of your CSV)
columns = [
    "time",
    "weight_0",
    "weight_1",
    "weight_total",
]

# Load CSV without header
df = pd.read_csv(file, header=None, names=columns)

# Plot all weight columns
for col in columns[2:]:  # skip date, time
    df[col] = pd.to_numeric(df[col], errors="coerce")
    plt.plot(df["time"], df[col], label=col)

# Labels
plt.xlabel("Time")
plt.ylabel("Weight")
plt.title("Weight Data")
plt.legend()

# Improve readability
plt.xticks(rotation=45)
plt.tight_layout()

plt.show()