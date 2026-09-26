from dotenv import load_dotenv

from services.source.live_packet_source import LivePacketSource
from services.source.pcap_file_source import PcapFilePacketSource

load_dotenv()

def main():
    file_source = PcapFilePacketSource("./dnp3.pcap", display_filter="dnp3")
    live_source = LivePacketSource("eth0")

    for packet in live_source.packets():
        print("New packet!")

if __name__ == '__main__':
    main()