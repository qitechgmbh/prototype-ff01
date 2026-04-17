import asyncio
from textual.screen import Screen
from textual.widgets import Label


class LiveTCPScreen(Screen):
    def compose(self):
        yield Label("Connecting...", id="out")

    async def on_mount(self):
        self.output = self.query_one("#out", Label)

        try:
            self.reader, self.writer = await asyncio.open_connection(
                "127.0.0.1", 55667
            )
        except Exception as e:
            self.output.update(f"Connection failed: {e}")
            return

        self.run_worker(self.read_loop)

    async def read_loop(self):
        self.output.update("Connected. Waiting for data...\n")

        while True:
            data = await self.reader.readline()
            if not data:
                break

            text = data.decode().strip()

            # ✅ This is safe inside worker in Textual
            self.output.update(text)

    def on_key(self, event):
        if event.key == "escape":
            self.app.pop_screen()