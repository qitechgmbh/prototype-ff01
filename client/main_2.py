from dash import Dash, html, dcc, Input, Output, callback, ctx
import plotly.express as px
import plotly.io as pio
import pandas as pd

import colors

# --------------------------
# SAMPLE DATA
# --------------------------
df = pd.read_csv(
    "https://raw.githubusercontent.com/plotly/datasets/master/gapminder_unfiltered.csv"
)

# fake "time" column for demo (00:00 - 23:59 simulation)
df["time_min"] = (df["year"] % 24) * 60


# --------------------------
# APP
# --------------------------
app = Dash(__name__)


# --------------------------
# LAYOUT
# --------------------------
app.layout = html.Div(
    [
        # TOP MODE SELECTOR
        dcc.Dropdown(
            id="mode",
            options=[
                {"label": "None",   "value": "none"  },
                {"label": "Live",   "value": "live"  },
                {"label": "Orders", "value": "orders"},
                {"label": "Days",   "value": "days"  },
            ],
            value="none",
            clearable=False
        ),

        html.Br(),

        # DYNAMIC CONTROLS
        html.Div(id="controls"),

        # html.Hr(),

        dcc.Graph(id="graph")
    ],
    className="app-root",
)


# --------------------------
# TIME FORMAT HELPER
# --------------------------
def fmt(mins):
    h = mins // 60
    m = mins % 60
    return f"{h:02d}:{m:02d}"


# --------------------------
# DYNAMIC CONTROLS
# --------------------------
@callback(
    Output("controls", "children"),
    Input("mode", "value")
)
def render_controls(mode):
    raise RuntimeError("WTF")

    # if mode == "none":
    #     return html.Div("")
# 
    # if mode == "live":
    #     return html.Div("Live mode: no filters")
# 
    # elif mode == "orders":
    #     return dcc.Dropdown(
    #         id="order-selector",
    #         options=[{"label": c, "value": c} for c in df.country.unique()],
    #         value="Canada",
    #         clearable=False
    #     )
# 
    # elif mode == "days":
    #     return html.Div([
    #         dcc.Dropdown(
    #             id="day-selector",
    #             options=[
    #                 {"label": str(y), "value": y}
    #                 for y in sorted(df.year.unique())
    #             ],
    #             value=df.year.min(),
    #             clearable=False
    #         ),
# 
    #         html.Br(),
# 
    #         dcc.RangeSlider(
    #             id="time-range",
    #             min=0,
    #             max=1439,
    #             step=5,
    #             value=[480, 1020],  # 08:00 → 17:00
    #             marks={
    #                 0: "00:00",
    #                 360: "06:00",
    #                 720: "12:00",
    #                 1080: "18:00",
    #                 1439: "23:59"
    #             }
    #         )
    #     ])


# --------------------------
# GRAPH UPDATE
# --------------------------
@callback(
    Output("graph", "figure"),
    Input("mode", "value"),
    Input("order-selector", "value"),
    Input("day-selector", "value"),
    Input("time-range", "value"),
)
def update_graph(mode, order, day, time_range):
    raise RuntimeError("WTF")

    dff = df.copy()

    # --------------------------
    # MODE LOGIC
    # --------------------------
    if mode == "live":
        dff = dff.tail(200)

    elif mode == "orders":
        if order:
            dff = dff[dff["country"] == order]

    elif mode == "days":
        if day:
            dff = dff[dff["year"] == day]

        if time_range:
            start, end = time_range
            dff = dff[dff["time_min"].between(start, end)]

    # --------------------------
    # PLOT
    # --------------------------
    # fig = px.line(
    #     dff,
    #     x="year",
    #     y="pop",
    #     color="country",
    #     title=f"Mode: {mode}"
    # )

    fig.update_layout(
        dragmode="pan",
        title=str(path.resolve()),
        xaxis_title="Time",
        yaxis_title="Weight",
        hovermode="x unified",
        template='catppuccin_mocha'
    )

    raise RuntimeError("WTF")

    # return fig


# --------------------------
if __name__ == "__main__":
    pio.templates.default = "catppuccin_mocha"

    app.run(debug=True)