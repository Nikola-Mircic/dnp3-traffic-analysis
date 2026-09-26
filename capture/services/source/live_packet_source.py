import pyshark

from domain.intrefaces.packet_source import PacketSource

class LivePacketSource(PacketSource):
    def __init__(self,
                 interface,
                 display_filter="dnp3"):
        print("Initializing LivePacketSource on {} for {}".format(interface, display_filter))
        self.interface = interface
        self.display_filter = display_filter
        self.capture = None

    def packets(self):
        print("Starting to sniff...")
        self.capture = pyshark.LiveCapture(
            interface=self.interface,
            display_filter=self.display_filter)

        try:
            for packet in self.capture.sniff_continuously():
                yield packet
        finally:
            self.capture.close()
            self.capture = None

    def stop(self):
        if self.capture is not None:
            self.capture.close()
            self.capture = None