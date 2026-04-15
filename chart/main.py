import sys
import pandas as pd
import plotly.graph_objects as go
import plotly.io as pio

import colors
import weights
import bounds
import states
import plates
import logs

path = sys.argv[1]

pio.templates.default = "catppuccin_mocha"

# Create figure
fig = go.Figure()

# Get first and last timestamps
df = weights.read(path)
first_ts = pd.to_datetime(df.iloc[0]["time"], format="mixed")
last_ts  = pd.to_datetime(df.iloc[-1]["time"], format="mixed")

bounds.init(path, fig, last_ts)
states.init(path, fig, last_ts)
weights.init(path, fig)
plates.init(path, fig, last_ts)
logs.init(path, fig)

# Layout
fig.update_layout(
    dragmode="pan",
    title=sys.argv[1],
    xaxis_title="Time",
    yaxis_title="Weight",
    hovermode="x unified"
)

fig.show(config = {'scrollZoom': True})