import struct
import zstandard as zstd
import pandas as pd

def read_batch(path):
    with open(path, "rb") as f:
        dctx = zstd.ZstdDecompressor()
        with dctx.stream_reader(f) as reader:
            data = reader.read()

    offset = 0

    (batch_id,) = struct.unpack_from("<Q", data, offset)
    offset += 8

    (prev_id_raw,) = struct.unpack_from("<Q", data, offset)
    offset += 8
    prev_id = None if prev_id_raw == 0 else prev_id_raw

    created_bytes = data[offset:offset + 23]
    offset += 23
    created = created_bytes.decode("utf-8", errors="ignore").rstrip("\x00")

    weights, offset = read_weights(data, offset)

    return {
        "id": batch_id,
        "prev_id": prev_id,
        "created": created,
        "weights": weights,
    }


def read_weights(data, offset):
    (length,) = struct.unpack_from("<H", data, offset)
    offset += 2

    dt = struct.unpack_from(f"<{length}H", data, offset)
    offset += 2 * length

    w0 = struct.unpack_from(f"<{length}H", data, offset)
    offset += 2 * length

    w1 = struct.unpack_from(f"<{length}H", data, offset)
    offset += 2 * length

    return {
        "len": length,
        "dt": dt,
        "w0": w0,
        "w1": w1,
    }, offset

def batch_to_df(batch):
    weights = batch["weights"]

    n = weights["len"]

    df = pd.DataFrame({
        "batch_id": [batch["id"]] * n,
        "created": [batch["created"]] * n,
        "dt": weights["dt"],
        "w0": weights["w0"],
        "w1": weights["w1"],
    })

    df["wt"] = df["w0"] + df["w1"]

    return df

if __name__ == "__main__":
    import sys

    path = sys.argv[1]

    batch = read_batch(path)

    df = batch_to_df(batch)

    print(df.head())

    print(df)