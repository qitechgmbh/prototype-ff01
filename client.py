import socket
import json
import time

HOST = "127.0.0.1"
PORT = 9000
HOME = "/home/entity/ff01/client"

def send_request(sock, req):
    msg = json.dumps(req) + "\n"
    sock.sendall(msg.encode("utf-8"))


def recv_response(sock):
    data = sock.recv(4096)
    if not data:
        return None
    return data.decode("utf-8")


def run_client():
    with socket.create_connection((HOST, PORT)) as sock:
        sock.settimeout(2.0)

        while True:
            try:
                send_request(sock, {"List": None})

                response = recv_response(sock)
                if response:
                    for line in response.strip().split("\n"):
                        if line:
                            try:
                                print(json.loads(line))
                            except json.JSONDecodeError:
                                print("RAW:", line)

                time.sleep(1)

            except socket.timeout:
                continue

            except ConnectionError:
                print("Connection closed")
                break


if __name__ == "__main__":
    run_client()