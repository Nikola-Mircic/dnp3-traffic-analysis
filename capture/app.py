import pyshark
from dotenv import load_dotenv

from services.pcap_file_source import PcapFilePacketSource

load_dotenv()

def main():
    source = PcapFilePacketSource("./dnp3.pcap", display_filter="dnp3")

    packets = []
    for packet in source.packets():
        packets.append(packet)


if __name__ == '__main__':
    main()