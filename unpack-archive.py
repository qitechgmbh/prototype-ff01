import io
import zipfile
from pathlib import Path
import pandas as pd
from pandas.errors import EmptyDataError
import shutil

REQUIRED_FILES = {
    "bounds.csv",
    "logs.csv",
    "orders.csv",
    "plates.csv",
    "states.csv",
    "weights.csv",
}


def deconstruct_and_save(zip_path, output_dir, name):
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    groups = {}

    with zipfile.ZipFile(zip_path, 'r') as z:
        folders, hours = validate_and_collect(z.namelist())

        print(f"Processing hours: {hours}")

        # -------------------------
        # READ FILES (raw)
        # -------------------------
        for hour in sorted(folders):
            for file in folders[hour]:
                path = f"{hour}/{file}"

                print(f"Reading {path}")

                with z.open(path) as f:
                    content = f.read()

                # normalize: strip trailing whitespace/newlines
                content = content.rstrip(b"\n\r")

                if not content:
                    print(f"Skipping empty file: {path}")
                    continue

                groups.setdefault(file, []).append(content)

    tmp_dir = output_dir / "tmp"

    # -------------------------
    # ENSURE ALL FILES EXIST
    # -------------------------
    for file in REQUIRED_FILES:
        out_path = tmp_dir / file
        out_path.parent.mkdir(parents=True, exist_ok=True)

        if not out_path.exists():
            print(f"Creating empty file: {file}")
            out_path.write_bytes(b"")

    # -------------------------
    # MERGE + SAVE
    # -------------------------
    for file, chunks in groups.items():
        out_path = tmp_dir / file

        print(f"Writing {file} ({len(chunks)} parts)")

        # ensure parent exists (safe guard)
        out_path.parent.mkdir(parents=True, exist_ok=True)

        with open(out_path, "wb") as out:
            if chunks:
                for i, chunk in enumerate(chunks):
                    if i > 0:
                        out.write(b"\n")
                    out.write(chunk)
            else:
                # ensure file exists even if empty
                out.write(b"")

        print(f"Saved {out_path}")


    orders = extract_orders(tmp_dir)

    # -------------------------
    # Install Orders
    # -------------------------
    order_dir = output_dir / "orders"
    order_dir.mkdir(parents=True, exist_ok=True)

    order_dir_tmp = order_dir / "tmp"
    order_dir_tmp.mkdir(parents=True, exist_ok=True)

    for id_, start, end in orders:
        for file in REQUIRED_FILES:
            src_path  = tmp_dir / file
            dest_path = order_dir_tmp / file

            state = "BEFORE"  # BEFORE → INSIDE → AFTER

            with open(src_path, "r", encoding="utf-8") as src, \
                open(dest_path, "w", encoding="utf-8") as dest:

                for line in src:
                    if not line.strip():
                        continue

                    timestamp = line.split(",", 1)[0].strip()

                    # -------------------------
                    # STATE TRANSITIONS
                    # -------------------------
                    if state == "BEFORE":
                        if timestamp >= start:
                            state = "INSIDE"
                        else:
                            continue

                    if state == "INSIDE":
                        if timestamp > end:
                            state = "AFTER"
                            break

                        dest.write(line)

        final_zip_path = order_dir / f"{id_}.zip"
        create_archive(order_dir_tmp, final_zip_path)


    print(f"Removing orders/tmp: {tmp_dir}")
    shutil.rmtree(order_dir_tmp, ignore_errors=True)

    print(f"Orders found {extract_orders(tmp_dir)}")

    final_zip_path = output_dir / "days" / f"{name}.zip"
    create_archive(tmp_dir, final_zip_path)

    print(f"Removing days/tmp: {tmp_dir}")
    shutil.rmtree(tmp_dir, ignore_errors=True)

def create_archive(tmp_dir, out_path):
    out_path.parent.mkdir(parents=True, exist_ok=True)

    with zipfile.ZipFile(out_path, "w", compression=zipfile.ZIP_DEFLATED) as z:
        for file_path in sorted(tmp_dir.iterdir()):
            # print(f"Adding {file_path.name}")
            z.write(file_path, arcname=file_path.name)

def validate_and_collect(namelist):
    folders = {}  # "HH" -> set(files)

    # -------------------------
    # SINGLE PASS
    # -------------------------
    for name in namelist:
        parts = name.split("/")

        if len(parts) != 2:
            raise RuntimeError(f"Invalid format: {name}")

        hour, filename = parts

        if not hour.isdigit() or len(hour) != 2:
            raise RuntimeError(f"Invalid hour format: {name}")

        h = int(hour)
        if not (0 <= h <= 23):
            raise ValueError(f"Hour out of range: {hour}")

        folders.setdefault(hour, set()).add(filename)

    if not folders:
        raise ValueError("No valid folders found")

    # -------------------------
    # VALIDATE HOURS (continuity)
    # -------------------------
    hours = sorted(int(h) for h in folders.keys())

    expected = list(range(hours[0], hours[-1] + 1))
    if hours != expected:
        missing = set(expected) - set(hours)
        raise ValueError(
            f"Non-continuous hours.\n"
            f"Found: {hours}\n"
            f"Missing: {sorted(missing)}"
        )

    # -------------------------
    # VALIDATE FILES PER FOLDER
    # -------------------------
    for hour, files in folders.items():
        missing = REQUIRED_FILES - files

        if missing: #ensure all required files are present
            extra = files - REQUIRED_FILES
            raise ValueError(
                f"Hour {hour} invalid files.\n"
                f"Missing: {sorted(missing)}\n"
                f"Extra (ignored): {sorted(extra)}"
            )

    return folders, hours

from pathlib import Path
from datetime import datetime


def extract_orders(tmp_dir):
    states_file = Path(tmp_dir) / "states.csv"

    if not states_file.exists():
        raise FileNotFoundError(f"Missing {states_file}")

    orders = {}  # id -> {"start": ts, "end": ts}

    with open(states_file, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()

            if not line:
                continue

            parts = [p.strip() for p in line.split(",")]

            if len(parts) != 3:
                continue  # or raise if strict

            ts_str, state_str, id_str = parts

            if id_str not in orders:
                orders[id_str] = {"start": None, "end": None}

            if state_str == "1":
                orders[id_str]["start"] = ts_str

            elif state_str == "2":
                orders[id_str]["end"] = ts_str

    # -------------------------
    # BUILD FINAL OUTPUT
    # -------------------------
    result = []

    for id_, data in orders.items():
        if data["start"] != None and data["end"] != None:
            result.append((id_, data["start"], data["end"]))

    return result


deconstruct_and_save("/home/entity/telemetry/Archive.zip", "/home/entity/qitech/telemetry", "20260420");