import base64
import io
from datetime import datetime, timedelta
from pathlib import Path
from dash import Dash, html, dcc, callback, Output, Input, State
import plotly.graph_objects as go
import plotly.express as px
import plotly.io as pio
import pandas as pd

from chart import open_chart

from colors import COLOR_PALETTE

base_path = Path.home() / "qitech" / "telemetry"

supported_machines = ["ff01", "ff02"]

machine_files = {}

now = datetime.now()
tomorrow = now + timedelta(days=1)

# archive / days

fig_layout = dict(
    paper_bgcolor="rgba(0,0,0,0)",
    plot_bgcolor="rgba(0,0,0,0)",
    font=dict(color=COLOR_PALETTE["text"], family="JetBrains Mono"),
    xaxis=dict(
        showgrid=True,
        gridcolor=COLOR_PALETTE["surface0"],
        gridwidth=1,
        linecolor="rgba(0,0,0,0)",
        zerolinecolor=COLOR_PALETTE["subtext0"],
    ),
    yaxis=dict(
        showgrid=True,
        gridcolor=COLOR_PALETTE["surface0"],
        gridwidth=1,
        linecolor="rgba(0,0,0,0)",
        zerolinecolor=COLOR_PALETTE["subtext0"],
    ),
    xaxis_title=None,
    yaxis_title=None,
    dragmode="pan"
);

fig_default = dict(layout=fig_layout);

app = Dash()
app.title = "Telemetry Client"

app.layout = [
    dcc.Interval(
        id="live-update",
            interval=int(1000),
            n_intervals=0,
    ),
    dcc.Interval(
        id="archive-update",
            interval=int(12_000),
            n_intervals=0,
    ),
    dcc.Graph(
        id="live-graph",
        config={"scrollZoom": True},
        # style={"display": "none"},
        figure=fig_default,
    ),
    dcc.Graph(
        id="archive-order-graph",
        config={"scrollZoom": True},
        style={"display": "none"},
        figure=fig_default,
    ),
    dcc.Graph(
        id="archive-day-graph",
        config={"scrollZoom": True},
        style={"display": "none"},
        figure=fig_default,
    ),
    dcc.Graph(
        id="import-graph",
        config={"scrollZoom": True},
        style={"display": "none"},
        figure=fig_default,
    ),
    html.Div(
        [
            dcc.Tabs([
                dcc.Tab(label="Live", children=[
                    dcc.Dropdown(
                        options=[],
                        value="ff01",
                        id="machine-selection",
                        searchable=False,
                        clearable=False,
                    ),
                ]),
                dcc.Tab(label="Archiv", children=[
                    dcc.Dropdown(
                        options=[],
                        value="ff01",
                        id="machine-selection-archive",
                        searchable=False,
                        clearable=False,
                    ),
                    dcc.Tabs([
                        dcc.Tab(label="Aufträge", children=[
                            #  dcc.Dropdown(
                            #      value=None,
                            #      id="order-selection",
                            #      searchable=True,
                            #      clearable=False,
                            #  ),
                        ]),
                        dcc.Tab(label="Tage", children=[
                            # dcc.Dropdown(
                            #     options=[
                            #         {"label": "16/04/26", "value": "20260416"},
                            #         {"label": "17/04/26", "value": "20260417"},
                            #         {"label": "18/04/26", "value": "20260418"},
                            #     ],
                            #     value="20260416",
                            #     id="day-selection",
                            #     searchable=True,
                            #     clearable=False,
                            #     placeholder="Suche"
                            # ),
                        ])
                    ], id="archive-tabs")
                ]),
                dcc.Tab(label="Import", children=[
                    dcc.Upload(
                        id="upload-data",
                        children=html.Button("Datei ablegen oder auswählen"),
                        multiple=False
                    )
                ])
            ], 
            id="main-tabs"
        )],
        id='app-root'
    ),
]

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
    Output('live-graph',          'style'),
    Output('archive-order-graph', 'style'),
    Output('archive-day-graph',   'style'),
    Output('import-graph',        'style'),
    Input('main-tabs',    'value'),
    Input('archive-tabs', 'value'),
)
def handle_mode_change(mode, mode_archive):
    if mode == "tab-1":
        # live view
        return mode_change_data(0)

    if mode == "tab-2":
        if mode_archive == "tab-1":
            # orders view
            return mode_change_data(1)
        else:
            # days view
            return mode_change_data(2)

    # import view
    return mode_change_data(3)

def mode_change_data(active_index: int, size: int = 4):
    hidden = {"display": "none"}
    shown  = {"display": "block"}

    styles = [hidden] * size
    styles[active_index] = shown

    return tuple(styles)

# @callback(
#     Output('archive-order-graph', 'figure'),
#     Input('order-selection', 'value'),
# )
# def update_graph_order(order_id):
#     fig = fig_default
#     return fig


@callback(
    Output('import-graph', 'figure'),
    Input('upload-data', 'contents'),
    State('upload-data', 'filename'),
)
def update_graph_import(data, name):
    if data == None:
        return fig_default

    content_type, content_string = data.split(',')

    decoded = base64.b64decode(content_string)

    file_stream = io.BytesIO(decoded)

    fig = open_chart(file_stream)
    fig.update_layout(fig_layout)

    return fig

archive_registry={}
archive_registry["ff01"] = {}
archive_registry["ff01"]["days"] = {}
archive_registry["ff01"]["orders"] = {}

archive_registry["ff02"] = {}
archive_registry["ff02"]["days"] = {}
archive_registry["ff02"]["orders"] = {}

@callback(
    Output('machine-selection-archive', "options"),
    Input('archive-update', 'n_intervals'),
)
def update_archive_lists(interval):
    reload_machine_files()

    options = []
    for machine_id, archive in machine_files.items():
        options.append({"label": machine_id.upper(), "value": machine_id},)



    return options

# @callback(
#     Output('graph-content', 'figure'),
#     Output('upload-data', 'contents'),
#     # Output('upload-data', 'filename'),
#     Input('machine-selection', 'value'),
#     Input('main-tabs', 'value'),
#     Input('archive-tabs', 'value'),
#     Input('order-selection', 'value'),
#     Input('day-selection', 'value'),
#     Input('upload-data', 'contents'),
#     State('upload-data', 'filename'),
# )
def update_graph(machine_id, menu_mode, archive_mode, order, day, import_data, import_name):

    print(import_data, import_name)

    if menu_mode == "tab-2":
        if archive_mode == "tab-1":
            return update_graph_order(machine_id, order)
        else: 
            return update_graph_day(machine_id, day)

    if menu_mode == "tab-3":
        return draw_import(import_data, import_name)

    return go.Figure(), None #, None, None


# @callback(
#     Output("order-selection", "options"),
#     Output("order-selection", "value"),
#     Input("machine-selection", "value")  # example dependency
# )
# def update_orders(machine):
#     # example: replace with your real logic
#     orders = machine_files[machine]["orders"]
# 
#     options = [
#         {"label": o, "value": o}
#         for o in orders
#     ]
# 
#     return options, orders[0] if orders else None


def reload_machine_files():
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

if __name__ == '__main__':
    app.run(debug=True)
