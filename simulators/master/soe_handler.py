from pydnp3 import opendnp3

def printPoint(header_info:opendnp3.HeaderInfo, datatype):
    """
    Print a point data in format:
     - "[EVENT/STATIC] data_type -> index : value"
    """
    def print_fnc(point):
        event_str = "EVENT" if header_info.isEventVariation else "STATIC"
        log_str = "[{}] {} -> {} : {}".format(event_str, opendnp3.GroupVariationToString(header_info.gv),point.index, point.value.value)
        print(log_str)

    return print_fnc

class SOEHandler(opendnp3.ISOEHandler):
    """
        Override ISOEHandler in this manner to implement application-specific sequence-of-events behavior.

        This is an interface for SequenceOfEvents (SOE) callbacks from the Master stack to the application layer.
    """

    def __init__(self):
        super(SOEHandler, self).__init__()

    def Process(self, info, values):
        """
            Process measurement data.

        :param info: HeaderInfo
        :param values: A collection of values received from the Outstation (various data types are possible).
        """
        values.ForeachItem(printPoint(info, type(values)))

    def Start(self):
        print('In SOEHandler.Start')

    def End(self):
        print('In SOEHandler.End')