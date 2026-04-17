import sys
import pandas as pd
import plotly.graph_objects as go
import plotly.io as pio

pio.templates["catppuccin_mocha"] = {
    "layout": {
        "paper_bgcolor": "#1e1e2e",
        "plot_bgcolor": "#1e1e2e",
        "font": {"color": "#cdd6f4"},
        "colorway": [
            "#89b4fa",  # blue
            "#f38ba8",  # red
            "#a6e3a1",  # green
            "#fab387",  # peach
            "#cba6f7",  # mauve
            "#94e2d5",  # teal
        ],
        "xaxis": {
            "gridcolor": "#313244",
            "zerolinecolor": "#6c7086",
        },
        "yaxis": {
            "gridcolor": "#313244",
            "zerolinecolor": "#6c7086",
        },
    }
}

COLOR_PALETTE = {
    "rosewater": "#f5e0dc",
    "flamingo": "#f2cdcd",
    "pink": "#f5c2e7",
    "mauve": "#cba6f7",
    "red": "#f38ba8",
    "maroon": "#eba0ac",
    "peach": "#fab387",
    "yellow": "#f9e2af",
    "green": "#a6e3a1",
    "teal": "#94e2d5",
    "sky": "#89dceb",
    "sapphire": "#74c7ec",
    "blue": "#89b4fa",
    "lavender": "#b4befe",

    "text": "#cdd6f4",
    "subtext0": "#a6adc8",
    "overlay0": "#6c7086",
    "surface0": "#313244",
    "base": "#1e1e2e",
    "mantle": "#181825",
    "crust": "#11111b",
}