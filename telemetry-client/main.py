from textual.app import App, ComposeResult
from textual.widgets import Header, Footer, Button, Input, ListView, ListItem, Label
from textual.containers import Vertical


class EntryApp(App):
    CSS = """
    Screen {
        align: center middle;
    }
    #menu {
        width: 40%;
        height: auto;
    }
    """

    def __init__(self):
        super().__init__()
        self.entries = []

    def compose(self) -> ComposeResult:
        yield Header()

        with Vertical(id="menu"):
            yield Button("Update Entries", id="update")
            yield Button("Display Entries", id="display")
            yield Button("Exit", id="exit")

        yield Footer()

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "update":
            self.show_input_screen()
        elif event.button.id == "display":
            self.show_entries_screen()
        elif event.button.id == "exit":
            self.exit()

    def show_input_screen(self):
        self.clear()
        self.mount(Header())

        self.input = Input(placeholder="Enter new item and press Enter")
        self.mount(self.input)
        self.input.focus()

        self.mount(Footer())

    def on_input_submitted(self, event: Input.Submitted) -> None:
        value = event.value.strip()
        if value:
            self.entries.append(value)

        self.return_to_menu()

    def show_entries_screen(self):
        self.clear()
        self.mount(Header())

        list_view = ListView()

        if not self.entries:
            list_view.append(ListItem(Label("No entries yet.")))
        else:
            for item in self.entries:
                list_view.append(ListItem(Label(item)))

        self.mount(list_view)
        self.mount(Footer())

    def key_escape(self):
        self.return_to_menu()

    def return_to_menu(self):
        self.clear()
        self.mount_all(self.compose())


if __name__ == "__main__":
    app = EntryApp()
    app.run()