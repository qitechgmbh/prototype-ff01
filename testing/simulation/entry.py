import random
import struct

def create_weights(dt) -> bytes: 
    size = 14

    # Metadata
    timestamp  = int(dt.timestamp() * 1_000_000) # as micros
    event_tag  = 0
    null_flags = 255

    # Data
    w0 = random.randint(0, 100)
    w1 = random.randint(0, 100)

    return struct.pack("<BQBBhh", size, timestamp, event_tag, null_flags, w0, w1)
