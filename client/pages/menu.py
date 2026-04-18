from textual.app import App, ComposeResult
from textual.widgets import Button
from textual.containers import Vertical
from textual.screen import Screen

from .live_data import LiveDataScreen
from .archive   import ArchivePage 

class MenuScreen(Screen):
    def compose(self) -> ComposeResult:
        with Vertical(id="menu"):
            yield Button("Live Data", id="live_data", classes="menu-btn")
            yield Button("Archive",   id="archive",   classes="menu-btn")
            yield Button("Exit",      id="exit",      classes="menu-btn")

    def on_button_pressed(self, event: Button.Pressed) -> None:
        app = self.app

        if event.button.id == "live_data":
            app.push_screen(LiveDataScreen())
        elif event.button.id == "archive":
            app.push_screen(ArchivePage())
        elif event.button.id == "exit":
            app.exit()