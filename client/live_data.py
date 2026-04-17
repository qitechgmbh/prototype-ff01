import asyncio
from textual.screen import Screen
from textual.widgets import Label, Log


class LiveTCPScreen(Screen):
    def compose(self):
        self.logger = Log()
        yield self.logger

    async def on_mount(self):
        self.logger.write_line(f"Connecting...")
        try:
            self.reader, self.writer = await asyncio.open_connection(
                "127.0.0.1", 55667
            )
        except Exception as e:
            self.logger.write_line(f"Connection failed: {e}")
            return

        self.run_worker(self.read_loop)

    async def read_loop(self):
        self.logger.write_line("Connected. Waiting for data...\n")

        while True:
            data = await self.reader.readline()
            if not data:
                break

            text = data.decode().strip()

            self.logger.write_line(text)

    def on_key(self, event):
        if event.key == "escape":
            self.app.pop_screen()