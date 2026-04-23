import pandas as pd

def read_df(data, name, columns):
    df = pd.read_csv(data, header=None, names=columns)

    # strip whitespace from all string columns
    for col in df.select_dtypes(include="object").columns:
        df[col] = df[col].str.strip()

    return df

def init_df(file_obj, name, columns):
    df = read_df(file_obj, name, columns)

    df["time"] = pd.to_datetime(df["time"], format="mixed")

    for col in columns[1:]:
        df[col] = pd.to_numeric(df[col], errors="coerce")

    return df


def extend(df, last_ts):
    if df.empty:
        return df

    # Copy last bounds row
    last_row = df.iloc[-1].copy()

    # Override time
    last_row["time"] = last_ts

    # Append
    df = pd.concat([df, last_row.to_frame().T], ignore_index=True)

    return df