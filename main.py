import asyncio
import os

import pyshark
from dotenv import load_dotenv

load_dotenv()

def listen():
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)

    capture = pyshark.LiveCapture(interface='Ethernet', bpf_filter="udp", eventloop=loop, tshark_path=os.getenv('TSHARK_PATH'))
    capture.apply_on_packets(print)

if __name__ == '__main__':
    listen()