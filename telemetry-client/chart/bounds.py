import pandas as pd
import plotly.graph_objects as go
import plotly.io as pio
import numpy as np

from .colors import COLOR_PALETTE
from .utils import extend, init_df

def init(path, fig, last_ts):
    columns = [
        "time",
        "min",
        "max",
        "desired",
        "trigger",
    ]

    colors = {
        "min":     "red",
        "max":     "red",
        "desired": "black",
        "trigger": "blue",
    }

    df = init_df(path, "bounds", columns)
    df = extend(df, last_ts=last_ts)

    fig.add_trace(
        go.Scatter(
            x=df["time"],
            y=df["trigger"],
            mode="lines",
            line=dict(
                color="rgba(0,0,0,0)",
                shape="hv",
            ),
            showlegend=False,
            legendgroup="bounds"
        )
    )

    fig.add_trace(
        go.Scatter(
            x=df["time"],
            y=df["min"],
            mode="lines",
            fill="tonexty",
            line=dict(
                color="rgba(0,0,0,0)",
                shape="hv",
            ),
            fillcolor="rgba(183, 189, 248, 0.65)",
            showlegend=False,
            legendgroup="bounds"
        )
    )

    fig.add_trace(
        go.Scatter(
            x=df["time"],
            y=df["max"],
            mode="lines",
            fill="tonexty",
            line=dict(
                color="rgba(0,0,0,0)",
                shape="hv",
            ),
            fillcolor="rgba(166, 218, 149, 0.65)",
            legendgroup="bounds",
            name="bounds"
        )
    )