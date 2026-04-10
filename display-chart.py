import sys
import pandas as pd
import plotly.express as px

if len(sys.argv) < 2:
    print("Usage: python plot_csv.py <file.csv>")
    sys.exit(1)

file = sys.argv[1]

# Load CSV
df = pd.read_csv(file)

# Change column names here if needed
x_col = df.columns[0]
y_col = df.columns[1]

# Create interactive plot
fig = px.line(df, x=x_col, y=y_col, title=f"{file}: {y_col} vs {x_col}")
fig.show()
