import pandas as pd
import plotly.graph_objects as go
import plotly.io as pio

from .colors import COLOR_PALETTE
from .utils import extend, init_df, read_df

def init(path, fig, last_ts):
    columns = [
        "time",
        "peak",
        "drop",
        "exit",
        "in_bounds",
    ]

    df = read_df(path, "plates", columns)
    df["time"] = pd.to_datetime(df["time"], format="mixed")
    for col in columns[1:3]:
        df[col] = pd.to_numeric(df[col], errors="coerce")

    df_true = df[df["in_bounds"] == "true"]
    df_false = df[df["in_bounds"] == "false"]

    fig.add_trace(init_collection(df_true,  "plates[in_bounds]", COLOR_PALETTE["green"]))
    fig.add_trace(init_collection(df_false, "plates[out_of_bounds]", COLOR_PALETTE["red"]))

def init_collection(df, name, color): 
    customdata = df[["peak", "drop", "exit", "in_bounds"]].copy()
    customdata["in_bounds"] = customdata["in_bounds"].map({"true": "Yes", "false": "No"})

    return go.Scatter(
        x=df["time"],
        y=df["peak"],
        mode="markers",
        marker=dict(
            size=10,
            symbol="diamond",
            color=color,
            line=dict(width=1, color="#1e1e2e")
        ),
        customdata=customdata,
        hovertemplate=(
            "Peak: %{customdata[0]}<br>"
            "Drop: %{customdata[1]}<br>"
            "Exit: %{customdata[2]}<br>"
            "In bounds: %{customdata[3]}<extra></extra>"
        ),
        name=name
    )