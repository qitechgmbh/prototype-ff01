import pandas as pd
import plotly.graph_objects as go

from colors import COLOR_PALETTE

def init(path, fig):
    columns = [
        "time",
        "type",
        "data",
    ]

    df = pd.read_csv(f"{path}/logs.csv", header=None, names=columns, usecols=[0, 1, 2])

    for col in df.select_dtypes(include="object").columns:
        df[col] = df[col].str.strip()

    df["time"] = pd.to_datetime(df["time"], format="mixed")

    warn = df[df["type"].str.upper() == "WARN"]
    err  = df[df["type"].str.upper() == "ERROR"]

    draw_collection(fig, warn, "warnings", COLOR_PALETTE["yellow"])
    draw_collection(fig, err,  "errors",   COLOR_PALETTE["red"])


def draw_collection(fig, df, name, color):
    fig.add_trace(
        go.Scatter(
            x=df["time"],
            y=[20] * len(df),
            mode="markers",
            marker=dict(
                size=12,
                symbol="arrow-up",
                color=color,
            ),
            customdata=df[["data"]],
            hovertemplate="%{customdata[0]}",
            name=name
        )
    )

    for t in df["time"]:
        fig.add_vline(
            x=t,
            line=dict(color=color, width=2)
        )