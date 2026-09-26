from dotenv import load_dotenv

from domain.models.dnp3_packet import FunctionCode
from services.adapters.pyshark_adapter import PySharkAdapter
from services.source.live_packet_source import LivePacketSource
from services.source.pcap_file_source import PcapFilePacketSource

load_dotenv()

def main():
    file_source = PcapFilePacketSource("./dnp3.pcap", display_filter="dnp3")
    live_source = LivePacketSource("eth0")

    adapter = PySharkAdapter()

    for packet in live_source.packets():
        result = adapter.adapt(packet)

        print("[{}] {} from {}/{}".format(result.captured_at,
                                       result.application.function_code.name,
                                       "MASTER" if result.data_link.control.dir else "OUTSTATION",
                                        result.data_link.source))

if __name__ == '__main__':
    main()
