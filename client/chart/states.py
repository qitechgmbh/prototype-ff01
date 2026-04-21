import pandas as pd
import plotly.graph_objects as go
import plotly.io as pio

from .colors import COLOR_PALETTE
from .utils import extend, init_df, read_df

# 2026-04-17T18:57:17.693528, 56569, 45066, 1.0, 128.0, 2026-4-17T18:17, 2026-4-17T18:57, 2400.00
# 2026-04-17T20:05:41.182901, 55785, 45066, 1.0, 0.0, 2026-4-17T19:32, 2026-4-17T20:5, 1980.00
# 2026-04-17T20:36:22.570856, 56571, 45066, 1.0, 0.0, 2026-4-17T20:6, 2026-4-17T20:36, 1800.00

def init(files, fig, last_ts):
    columns = [
        "time",
        "state_id",
        "order_id",
    ]

    df = init_df(files["states.csv"], "states", columns)
    df = extend(df, last_ts=last_ts)

    df["active"] = df["state_id"] != 0

    df["group"] = (df["active"] != df["active"].shift()).cumsum()

    for _, grp in df.groupby("group"):
        if grp["active"].iloc[0]:  # only active states

            start = grp["time"].iloc[0]
            end = grp["time"].iloc[-1]

            order_id = grp["order_id"].iloc[0]

            fig.add_vrect(
                x0=start,
                x1=end,
                fillcolor="rgba(49, 50, 68, 0.5)",  # Catppuccin yellow subtle
                layer="below",
                line_width=0
            )

            # midpoint for label
            mid = start + (end - start) / 2

            # label
            fig.add_annotation(
                x=mid,
                y=1,
                text=f"Order {order_id}",
                showarrow=False,
                yref="paper",  # relative to full chart height
                font=dict(color="#cdd6f4", size=12),
                align="center"
            )