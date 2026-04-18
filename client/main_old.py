# from textual.app import App
# 
# from pages.menu import MenuScreen
# 
# class EntryApp(App):
#     CSS_PATH = "styles.tcss"
# 
#     def __init__(self):
#         super().__init__()
#         self.entries = []
# 
#     def on_mount(self):
#         self.theme = "catppuccin-macchiato"
#         self.push_screen(MenuScreen())
# 
# if __name__ == "__main__":
#     EntryApp().run()


app.layout = html.Div([
    html.Div([
        html.Button("A", id="btn-a", n_clicks=0),
        html.Button("B", id="btn-b", n_clicks=0),
    ]),
    dcc.Graph(id="chart")
])

@app.callback(
    Output("chart", "figure"),
    Input("btn-a", "n_clicks"),
    Input("btn-b", "n_clicks"),
)
def update(a, b):
    ctx = dash.callback_context

    if not ctx.triggered:
        category = "A"
    else:
        button_id = ctx.triggered[0]["prop_id"].split(".")[0]
        category = "A" if button_id == "btn-a" else "B"

    filtered = df[df["category"] == category]
    return px.line(filtered, x="time", y="value")