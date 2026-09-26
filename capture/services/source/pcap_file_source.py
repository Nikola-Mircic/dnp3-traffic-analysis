from domain.intrefaces.packet_source import PacketSource
import pyshark

class PcapFilePacketSource(PacketSource):
    def __init__(self,
                 file_path: str,
                 display_filter= "dnp3"):
        self.file_path = file_path
        self.display_filter = display_filter

    def packets(self):
        capture = pyshark.FileCapture(
            self.file_path,
            display_filter=self.display_filter,
            include_raw=True,
            use_ek=True,
            use_json=True)
        try:
            for packet in capture:
                yield packet
        finally:
            capture.close()