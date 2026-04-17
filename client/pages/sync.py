from pathlib import Path
from datetime import datetime

from textual.screen import Screen
from textual.widgets import Header, Footer, Label, ListView, ListItem, Button, Log
from textual.containers import Horizontal, Vertical

import zipfile
import io
import asyncio
from datetime import datetime

from utils.archive_parser import deconstruct_and_save

class SyncPage(Screen):
    def __init__(self):
        super().__init__()
        self.base_path = Path.home() / "qitech" / "telemetry" / "ff01"
        self.base_path.mkdir(parents=True, exist_ok=True)
        self.logger = None

    def compose(self):
        self.logger = Log()
        yield self.logger

    def on_mount(self):
        self.run_worker(self.synchronize, exclusive=True)

    async def synchronize(self):
        entries = await self.fetch_entry_ids()

        for entry_id in entries: 
            self.logger.write_line(f"Retrieving Entry: {entry_id}")

            result = await self.fetch_entry(entry_id)

            if not result["ok"]:
                self.logger.write_line(f"Operation Failed: {result['error']}")
                self.app.pop_screen()
                return

            deconstruct_and_save(result["value"], self.base_path, entry_id, self.logger)

        self.logger.write_line(f"Synchronizing was successfull!")
        self.logger.write_line(f"Press ESC key to return")

    async def fetch_entry(self, id: str):
        reader, writer = await asyncio.open_connection(
            "127.0.0.1", 25565
        )

        try:
            # -------------------------
            # send request (FIXED)
            # -------------------------
            writer.write(f"GET {id}\n".encode())
            await writer.drain()

            # -------------------------
            # read all response
            # -------------------------
            raw = await reader.read()

        finally:
            writer.close()
            await writer.wait_closed()

        # -------------------------
        # detect error vs binary
        # -------------------------
        if raw.startswith(b"[Error]"):
            return {
                "ok": False,
                "error": raw.decode(errors="ignore").strip()
            }

        return {
            "ok": True,
            "value": raw
        }

    async def fetch_entry_ids(self):
        try:
            entries = []

            reader, writer = await asyncio.open_connection(
                "127.0.0.1", 25565
            )

            writer.write(b"LIST ALL\n")
            await writer.drain()

            # read response (line-based)
            raw = await reader.read()  # or read until EOF

            writer.close()
            await writer.wait_closed()

            text = raw.decode().strip()

            for line in text.splitlines():
                line = line.strip()

                if len(line) == 8 and line.isdigit():
                    try:
                        entries.append(line)
                    except ValueError:
                        pass
            
            return entries

        except Exception as e:
            raise RuntimeError(e)

    def on_key(self, event):
        if event.key == "escape":
            self.app.pop_screen()