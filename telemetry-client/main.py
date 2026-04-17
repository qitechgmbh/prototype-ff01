from textual.app import App, ComposeResult
from textual.widgets import Header, Footer, Button, Input, ListView, ListItem, Label
from textual.containers import Vertical
from textual.screen import Screen

import live_data
import archive  

class MenuScreen(Screen):
    def compose(self) -> ComposeResult:
        yield Header()
        with Vertical(id="menu"):
            yield Button("Live Data", id="live_data")
            yield Button("Archive",   id="archive")
            yield Button("Exit",      id="exit")
        yield Footer()

    def on_button_pressed(self, event: Button.Pressed) -> None:
        app = self.app

        if event.button.id == "live_data":
            app.push_screen(live_data.LiveTCPScreen())
        elif event.button.id == "archive":
            app.push_screen(archive.ArchivePage())
        elif event.button.id == "exit":
            app.exit()

class EntryApp(App):
    def __init__(self):
        super().__init__()
        self.entries = []

    def on_mount(self):
        self.push_screen(MenuScreen())


if __name__ == "__main__":
    EntryApp().run()