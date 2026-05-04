import time
import pycurl
import binascii
from io import BytesIO

def get_live():
    url = "http://localhost:9000/api/telemetry/live"
    
    buffer = BytesIO()

    c = pycurl.Curl()
    c.setopt(c.URL, url)
    c.setopt(c.WRITEDATA, buffer)

    try:
        c.perform()

        # Check for response code
        response_code = c.getinfo(c.RESPONSE_CODE)
        if response_code != 200:
            print(f"Request failed with status code: {response_code}")
            return

        # Get all the raw byte data from the buffer
        data = buffer.getvalue()

        # Initialize event count
        event_count = 0
        index = 0

        while index < len(data):
            event_size = data[index]
            index += event_size + 1
            event_count += 1

        print(f"Total number of events: {event_count}")
    
    except pycurl.error as e:
        print(f"HTTP get_live failed: {e}")
    
    finally:
        c.close()

def main():
    while True:
        get_live()
        time.sleep(0.25)

if __name__ == "__main__":
    main()