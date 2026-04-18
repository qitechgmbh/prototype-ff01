from pathlib import Path
from datetime import datetime

from textual.screen import Screen
from textual.widgets import Header, Footer, Label, ListView, ListItem, Button
from textual.containers import Horizontal, Vertical

import zipfile
import io
import asyncio
from datetime import datetime

from pages.sync import SyncPage

from chart import open_chart

class ArchivePage(Screen):
    def __init__(self):
        super().__init__()

        base_path = Path.home() / "qitech" / "telemetry" / "ff01"

        self.base_path_days = base_path / "days"
        self.base_path_days.mkdir(parents=True, exist_ok=True)
        self.days = []

        self.base_path_orders = base_path / "orders"
        self.base_path_orders.mkdir(parents=True, exist_ok=True)
        self.orders = []

    def compose(self):
        with Vertical():
            with Horizontal():
                with Vertical():
                    yield Label("Orders")
                    self.orders_list = ListView()
                    yield self.orders_list

                with Vertical():
                    yield Label("Days")
                    self.days_list = ListView()
                    yield self.days_list

            yield Button("Synchronize", id="synchronize", classes="sync-btn")

    def on_mount(self):
        self.refresh_data()

    def on_screen_resume(self):
        self.refresh_data()

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "synchronize":
            self.app.push_screen(SyncPage())

    def refresh_data(self):
        self.update_orders()
        self.update_days()
        self.update_ui()

    def update_days(self):
        self.days.clear()

        if not self.base_path_days.exists():
            return

        for zip_file in self.base_path_days.iterdir():
            if not zip_file.is_file():
                continue

            if zip_file.suffix != ".zip":
                continue

            name = zip_file.stem  # removes ".zip" → "20260416"

            if len(name) == 8 and name.isdigit():
                self.days.append(name)
        
        self.days.sort(reverse=True)

    def update_orders(self):
        self.orders.clear()

        if not self.base_path_orders.exists():
            return

        for zip_file in self.base_path_orders.iterdir():
            if not zip_file.is_file():
                continue

            if zip_file.suffix != ".zip":
                continue

            name = zip_file.stem  # removes ".zip" → "20260416"

            if name.isdigit():
                self.orders.append(name)
        
        self.orders.sort(reverse=True)

    def update_ui(self):
        self.days_list.clear()

        for day in self.days:
            dt = datetime.strptime(day, "%Y%m%d")
            label_text = dt.strftime("%d/%m/%y")

            item = ListItem(Label(label_text))
            item.ref_value = day
            self.days_list.append(item)

        self.orders_list.clear()

        for order in self.orders:
            label_text = order

            item = ListItem(Label(label_text))
            item.ref_value = order
            self.orders_list.append(item)

    def on_key(self, event):
        if event.key == "escape":
            self.app.pop_screen()

    def on_list_view_selected(self, event: ListView.Selected) -> None:
        if event.list_view is self.days_list:
            day = event.item.ref_value
            sub_path = day + ".zip"
            open_chart(self.base_path_days / sub_path)

        if event.list_view is self.orders_list:
            id_ = event.item.ref_value
            sub_path = id_ + ".zip"
            open_chart(self.base_path_orders / sub_path)