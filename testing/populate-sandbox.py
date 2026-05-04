import os
from pathlib import Path
import subprocess
import shutil
import random
import struct
from datetime import datetime, timedelta, timezone

exe_path = Path(__file__).resolve()

# ../sandbox/logs relative to script
log_dir = (exe_path.parent / "../sandbox/logs").resolve()

# 1) create or clear logs dir
if log_dir.exists():
    shutil.rmtree(log_dir)

log_dir.mkdir(parents=True, exist_ok=True)

print("Logs directory:", log_dir)

# 2) today (UTC)
today = datetime.now().replace(hour=0, minute=0, second=0, microsecond=0)

# 3) generate files from today back 7 days
# Number of days to generate (last 7 days)
for i in range(0, 7):
    day = today - timedelta(days=i)

    file_path = log_dir / f"{day.strftime('%Y%m%d')}.wal"
    print("Creating:", file_path)

    with open(file_path, "wb") as f:
        # start timestamp (midnight of the day in microseconds since epoch)
        day_start = datetime.combine(day, datetime.min.time())
        ts = int(day_start.timestamp() * 1_000_000)

        # Generate a random number of events for the day
        num_events = random.randint(50, 150)

        # Time span for the entire day in seconds (24 hours)
        total_seconds_in_day = 86400

        # Calculate the time difference between events to spread them evenly
        time_increment = total_seconds_in_day // num_events

        # Generate events sequentially across the day
        for _ in range(num_events):
            size = 14

            # Calculate the next timestamp by adding the time increment
            ts += time_increment * 1_000_000  # Convert to microseconds

            tag = 0
            null_flags = 255
            w0  = random.randint(0, 100)
            w1  = random.randint(0, 100)

            # Write the event to the file
            f.write(struct.pack("<BQBBhh", size, ts, tag, null_flags, w0, w1))


# 4. Run all tests first
server_path = exe_path.parent / "../telemetry-server/Cargo.toml"

env = os.environ.copy()
env["DIR_LOGS"]    = str((exe_path.parent / "../sandbox/logs").resolve())
env["DIR_ARCHIVE"] = str((exe_path.parent / "../sandbox/archive").resolve())
env["INGEST_PORT"] = "9000"
env["HTTP_PORT"]   = "9001"

subprocess.run(
    ["cargo", "run", "--manifest-path", str(server_path.resolve())],
    env=env,
    check=True
)

# Run curl to fetch data from the /telemetry/live endpoint
def fetch_live_data(url: str):
    result = subprocess.run(
        ["curl", "-s", url],
        capture_output=True,  # Capture stdout and stderr
        text=True,            # Get the output as a string
    )

    # Check if the curl command was successful
    if result.returncode == 0:
        return result.stdout  # Return the raw data
    else:
        raise Exception(f"Error fetching live data: {result.stderr}")

# Example usage:
url = "http://localhost:9001/telemetry/live"
raw_data = fetch_live_data(url)

def replay(raw_data):
    # Ensure the data is not empty
    if len(raw_data) > 0:
        # Read the first byte, which indicates the payload size
        first_byte = raw_data[0]

        # Print the first byte as a hex value
        print(f"First byte (payload size): {first_byte:02x}")
    else:
        print("No data received!")

# Example usage
raw_data = fetch_live_data("http://localhost:9001/telemetry/live")
replay(raw_data)