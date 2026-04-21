import sys
import io
import pandas as pd
import plotly.graph_objects as go
import plotly.io as pio
import zipfile

from . import colors
from . import weights
from . import bounds
from . import states
from . import plates
from . import logs

REQUIRED_FILES = {
    "bounds.csv",
    "logs.csv",
    "orders.csv",
    "plates.csv",
    "states.csv",
    "weights.csv",
}

def open_chart(file_stream):
    files = extract_files(file_stream)

    columns = [
        "time",
        "weight_0",
        "weight_1",
        "weight_total",
    ]

    pio.templates.default = "catppuccin_mocha"

    # Create figure
    fig = go.Figure()

    # Get first and last timestamps
    df = weights.read(files["weights.csv"])
    first_ts = pd.to_datetime(df.iloc[0]["time"], format="mixed")
    last_ts  = pd.to_datetime(df.iloc[-1]["time"], format="mixed")

    files["weights.csv"].seek(0)

    bounds .init(files["bounds.csv"],  fig, last_ts)
    states .init(files, fig, last_ts)
    weights.init(files["weights.csv"], fig)
    plates .init(files["plates.csv"],  fig, last_ts)
    logs   .init(files["logs.csv"],    fig)

    # Layout
    # fig.update_layout(
    #     dragmode="pan",
    #     title=str(path.resolve()),
    #     xaxis_title="Time",
    #     yaxis_title="Weight",
    #     hovermode="x unified"
    # )

    return fig
    # fig.show(config = {'scrollZoom': True})


def extract_files(file_stream):
    file_buffers = {}

    with zipfile.ZipFile(file_stream, "r") as z:
        zip_files = set(z.namelist())

        missing = REQUIRED_FILES - zip_files
        extra = zip_files - REQUIRED_FILES

        if missing:
            raise ValueError(f"Missing files in zip: {missing}")
        if extra:
            raise ValueError(f"Unexpected files in zip: {extra}")

        for file in REQUIRED_FILES:
            with z.open(file) as f:
                text = f.read().decode("utf-8")
                file_buffers[file] = io.StringIO(text)

    return file_buffers