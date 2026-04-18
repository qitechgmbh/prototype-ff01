from pathlib import Path
from dash import Dash, html, dcc, callback, Output, Input, State
import plotly.graph_objects as go
import plotly.express as px
import pandas as pd

from chart import open_chart

base_path = Path.home() / "qitech" / "telemetry"

supported_machines = ["ff01", "ff02"]

machine_files = {}

for machine in supported_machines: 
    machine_files[machine] = {
        "orders": [],
        "days": []
    }

    # collect days
    days_dir = base_path / machine / "days"
    days_dir.mkdir(parents=True, exist_ok=True)
    for zip_file in days_dir.iterdir():
        if not zip_file.is_file():
            continue

        if zip_file.suffix != ".zip":
            continue

        name = zip_file.stem

        if name.isdigit():
            machine_files[machine]["days"].append(name)

    # collect orders
    orders_dir = base_path / machine / "orders"
    orders_dir.mkdir(parents=True, exist_ok=True)
    for zip_file in orders_dir.iterdir():
        if not zip_file.is_file():
            continue

        if zip_file.suffix != ".zip":
            continue

        name = zip_file.stem  # removes ".zip" → "20260416"

        if name.isdigit():
            machine_files[machine]["orders"].append(name)


app = Dash()
app.title = "Telemetry Client"

app.layout = [
    # html.H1(children='QiTech Telemetry Client v0.1', style={'textAlign':'left'}),
    dcc.Graph(
        id="graph-content",
        config={"scrollZoom": True}
    ),
    html.Div(
        [
            dcc.Dropdown(
                options=[
                    {"label": "FF01", "value": "ff01"},
                    {"label": "FF02", "value": "ff02"},
                ],
                value="ff01",
                id="machine-selection",
                searchable=False,
                clearable=False,
            ),
            dcc.Tabs([
                dcc.Tab(label="Live", children=[
                    
                ]),
                dcc.Tab(label="Archiv", children=[
                    dcc.Tabs([
                        dcc.Tab(label="Aufträge", children=[
                            dcc.Dropdown(
                                value=None,
                                id="order-selection",
                                searchable=True,
                                clearable=False,
                            ),
                        ]),
                        dcc.Tab(label="Tage", children=[
                            dcc.Dropdown(
                                options=[
                                    {"label": "16/04/26", "value": "20260416"},
                                    {"label": "17/04/26", "value": "20260417"},
                                    {"label": "18/04/26", "value": "20260418"},
                                ],
                                value="20260416",
                                id="day-selection",
                                searchable=True,
                                clearable=False,
                                placeholder="Suche"
                            ),
                        ])
                    ], id="archive-tabs")
                ]),
                dcc.Tab(label="Import", children=[
                    dcc.Upload(
                        id="upload-data",
                        children=html.Button("Datei Auswählen"),
                        multiple=False
                    )
                ])
            ], 
            id="main-tabs"
        )],
        id='app-root'
    ),
]

def update_graph_order(machine, order):
    name = order + ".zip"
    path = base_path / machine / "orders" / name
    fig  = open_chart(path)

    fig.update_layout(
        template="plotly_dark",
        paper_bgcolor="rgba(0,0,0,0)",
        plot_bgcolor="rgba(0,0,0,0)",
        font=dict(color="white", family="JetBrains Mono"),
        xaxis_title=None,
        yaxis_title=None,
        dragmode="pan"
    )

    return fig

def update_graph_day(machine, day):
    name = day + ".zip"
    path = base_path / machine / "days" / name
    fig  = open_chart(path)

    fig.update_layout(
        template="plotly_dark",
        paper_bgcolor="rgba(0,0,0,0)",
        plot_bgcolor="rgba(0,0,0,0)",
        font=dict(color="white", family="JetBrains Mono"),
        xaxis_title=None,
        yaxis_title=None,
        dragmode="pan"
    )

    return fig

@callback(
    Output('graph-content', 'figure'),
    Input('machine-selection', 'value'),
    Input('main-tabs', 'value'),
    Input('archive-tabs', 'value'),
    Input('order-selection', 'value'),
    Input('day-selection', 'value'),
)
def update_graph(machine, menu_mode, archive_mode, order, day):

    if menu_mode == "tab-2":
        if archive_mode == "tab-1":
            return update_graph_order(machine, order)
        else: 
            return update_graph_day(machine, day)

    return go.Figure()


@callback(
    Output("order-selection", "options"),
    Output("order-selection", "value"),
    Input("machine-selection", "value")  # example dependency
)
def update_orders(machine):
    # example: replace with your real logic
    orders = machine_files[machine]["orders"]

    options = [
        {"label": o, "value": o}
        for o in orders
    ]

    return options, orders[0] if orders else None

@callback(
    Output('graph-content', 'figure'),
    Input('upload-data', 'contents'),
    State('upload-data', 'filename'),
    State('upload-data', 'last_modified')
    )
def on_file_selected(filename):

    raise RuntimeError(filename)

    fig = open_chart(filename)

    fig.update_layout(
        template="plotly_dark",
        paper_bgcolor="rgba(0,0,0,0)",
        plot_bgcolor="rgba(0,0,0,0)",
        font=dict(color="white", family="JetBrains Mono"),
        xaxis_title=None,
        yaxis_title=None,
        dragmode="pan"
    )

    return fig

if __name__ == '__main__':
    app.run(debug=True)
