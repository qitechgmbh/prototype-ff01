import socket
import asyncio
import os
from datetime import datetime

import entry

SOCKET_PATH = "/tmp/events.sock"

clients = set()

async def handle_client(conn):
    clients.add(conn)
    try:
        while True:
            await asyncio.sleep(3600)  # keep connection alive
    finally:
        clients.remove(conn)
        conn.close()

async def accept_clients(server_sock):
    loop = asyncio.get_running_loop()
    while True:
        conn, _ = await loop.sock_accept(server_sock)
        conn.setblocking(False)
        asyncio.create_task(handle_client(conn))

async def send_broadcast(interval):
    while True:
        data = entry.create_weights(datetime.now())

        dead_clients = []
        for c in clients:
            try:
                c.sendall(data)
            except Exception:
                dead_clients.append(c)

        for c in dead_clients:
            clients.discard(c)
            c.close()

        print(f"Sending: {data.hex()} to {len(clients)} clients")

        await asyncio.sleep(interval)

async def main():
    if os.path.exists(SOCKET_PATH):
        os.remove(SOCKET_PATH)

    server_sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    server_sock.bind(SOCKET_PATH)
    server_sock.listen()
    server_sock.setblocking(False)

    await asyncio.gather(
        accept_clients(server_sock),
        send_broadcast(0.5)
    )

if __name__ == "__main__":
    asyncio.run(main())