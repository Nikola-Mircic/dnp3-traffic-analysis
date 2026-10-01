from dotenv import load_dotenv
from pydnp3 import opendnp3

from domain.models.dnp3_packet import PointFlags
from services.adapters.pyshark_adapter import PySharkAdapter
from services.source.live_packet_source import LivePacketSource
from services.source.pcap_file_source import PcapFilePacketSource

load_dotenv()

def _quality_to_str(quality: PointFlags|None):
    if quality is None:
        return ""

    str_quality = []
    if quality.online:
        str_quality.append("ONLINE")
    if quality.restart:
        str_quality.append("RESTART")
    if quality.comm_lost:
        str_quality.append("COMM_LOST")
    if quality.local_forced:
        str_quality.append("LOCAL_FORCE")
    if quality.remote_forced:
        str_quality.append("REMOTE_FORCE")
    if quality.chatter_filter:
        str_quality.append("CHATER_FILTER")
    if quality.rollover:
        str_quality.append("ROLLOVER")
    if quality.discontinuity:
        str_quality.append("DISCONTINUITY")
    if quality.overrange:
        str_quality.append("OVERRANGE")
    if quality.ref_error:
        str_quality.append("REF_ERROR")

    return ",".join(str_quality)

def main():
    file_source = PcapFilePacketSource("./dnp3.pcap")
    live_source = LivePacketSource("eth0")

    adapter = PySharkAdapter()

    for packet in file_source.packets():
        result = adapter.adapt(packet)
        order_str = ""
        if result.transport.first:
            order_str = "FIRST"
        if result.transport.final:
            order_str = "FINAL"
        if result.transport.first and result.transport.final:
            order_str = "FIRST AND FINAL"

        print("---")
        print("#{} {}".format(result.transport.sequence, order_str))
        print("[{}] {} from {}/{}".format(result.captured_at,
                                          result.application.function_code.name,
                                          "MASTER" if result.data_link.control.dir else "OUTSTATION",
                                          result.data_link.source))
        for obj in result.objects:
            group = obj.group
            variation = obj.variation

            gv_name = opendnp3.GroupVariationToString(opendnp3.GroupVariationFromType(group<<8 | variation))
            print("Object - {} [{}.{}], {} points{}".format(gv_name, group, variation, len(obj.points), "." if len(obj.points) == 0 else ":"))
            for point in obj.points:
                print(" - {}: {} {}".format(point.index, point.value, _quality_to_str(point.flags)))

        print("---")



if __name__ == '__main__':
    main()
