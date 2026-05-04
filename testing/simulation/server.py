import time
import subprocess
from pathlib import Path
import os

base_dir = Path(__file__).resolve().parent

# go up one level from this file's directory
sandbox_dir = base_dir.parent / "sandbox"

dir_logs = sandbox_dir / "logs"
dir_archive = sandbox_dir / "archive"

env = os.environ.copy()
env["DIR_LOGS"]    = str(dir_logs)
env["DIR_ARCHIVE"] = str(dir_archive)
env["INGEST_PORT"] = str(0)
env["HTTP_PORT"]   = str(9000)

project_dir = base_dir.parents[1] / "telemetry-server"

time.sleep(0.25)

subprocess.run(
    ["cargo", "run"],
    cwd=project_dir,
    env=env,
    check=True
)