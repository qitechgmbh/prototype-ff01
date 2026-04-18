from dash import Dash, html, dcc, callback, Output, Input
import plotly.express as px
import pandas as pd

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
                                options=[
                                    {"label": "56569", "value": "56569"},
                                    {"label": "55608", "value": "55608"},
                                ],
                                value="56569",
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
                    ])
                ]),
                dcc.Tab(label="Import", children=[
                    dcc.Upload(
                        id="upload-data",
                        children=html.Button("Datei Auswählen"),
                        multiple=False
                    )
                ])
            ]),
            # dcc.Graph(id='graph-content')
        ],
        id='app-root'
    ),
]

@callback(
    Output('graph-content', 'figure'),
    Input('machine-selection', 'value')
)
def update_graph(value):
# Custom data (3 entries)
    dff = pd.DataFrame({
        "year": [2000, 2010, 2020],
        "pop": [5, 7, 9]
    })

    value = "Sample Data"

    fig = px.line(
        dff,
        x="year",
        y="pop",
    )

    fig.update_layout(
        template="plotly_dark",
        paper_bgcolor="rgba(0,0,0,0)",
        plot_bgcolor="rgba(0,0,0,0)",
        font=dict(color="white", family="JetBrains Mono"),
        xaxis_title=None,
        yaxis_title=None,
        dragmode="pan"
    )

    fig.update_traces(
        line=dict(color="#00d4ff", width=3)
    )

    return fig

if __name__ == '__main__':
    app.run(debug=True)
