import pandas as pd
import plotly.graph_objects as go
import plotly.io as pio

from colors import COLOR_PALETTE
from utils  import extend, init_df, read_df

def read(path):
    columns = [
        "time",
        "weight_0",
        "weight_1",
        "weight_total",
    ]

    return read_df(path, "weights", columns)


def init(path, fig):
    columns = [
        "time",
        "weight_0",
        "weight_1",
        "weight_total",
    ]

    colors = {
        "weight_0":     COLOR_PALETTE["peach"],
        "weight_1":     COLOR_PALETTE["blue"],
        "weight_total": COLOR_PALETTE["text"],
    }

    df = read(path)

    for col in columns[1:]:
        fig.add_trace(
            go.Scatter(
                x=df["time"],
                y=df[col],
                mode="lines",
                name=col,
                line=dict(color=colors[col])
            )
        )

    return df["time"].iloc[0], df["time"].iloc[-1]