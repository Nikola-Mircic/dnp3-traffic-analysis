from domain.models.dnp3_packet import PointFlags, DNP3Point

PYSHARK_ANALOG_VALUE_FIELD = "al_ana_int"
PYSHARK_BINARY_VALUE_FIELD = "al_biq_b7"
PYSHARK_COUNTER_VALUE_FIELD = "al_cnt"

def _list_to_quality_obj(val_list, binary=False, analog=False, counter=False):
    point = PointFlags(
        online=val_list[0],
        restart=val_list[1],
        comm_lost=val_list[2],
        remote_forced=val_list[3],
        local_forced=val_list[4],
        chatter_filter=False,
        overrange=False,
        ref_error=False,
        rollover=False,
        discontinuity=False,
    )
    point.online = val_list[0]
    point.restart = val_list[1]
    point.comm_lost = val_list[2]
    point.remote_forced = val_list[3]
    point.local_forced = val_list[4]
    if binary:
        point.chatter_filter = val_list[5]

    if analog:
        point.overrange = val_list[5]
        point.ref_error = val_list[6]

    if counter:
        point.rollover = val_list[5]
        point.discontinuity = val_list[6]

    return point

def BuildQualityParser(quality_field_prefix):
    def parser_func(layer):
        # parse list of quality bits for each point
        qualitiy_bits_values = []
        for i in range(8):
            qualitiy_bits_values.append([f.int_value == 1 for f in layer.get_field(f"al_{quality_field_prefix}q_b{i}").all_fields])

        qualitiy_bits_values = list(zip(*qualitiy_bits_values))

        return [_list_to_quality_obj(bits) for bits in qualitiy_bits_values]

    return parser_func

def getBinaryValues(layer):
    return [f.int_value == 1 for f in layer.get_field(PYSHARK_BINARY_VALUE_FIELD).all_fields]

def getAnalogIntValues(layer):
    return [int(f.show) for f in layer.get_field(PYSHARK_ANALOG_VALUE_FIELD).all_fields]

def getCounterValues(layer):
    return [int(f.show) for f in layer.get_field(PYSHARK_COUNTER_VALUE_FIELD).all_fields]

def ParsePoints(group, var, layer):
    parser = None
    values = []

    match group:
        case 1 | 2: # Binary Inputs
            values = getBinaryValues(layer)
            parser = BuildQualityParser("bi")
        case 30 | 32: # Analog Inputs
            values = getAnalogIntValues(layer)
            parser = BuildQualityParser("ai")
        case 20 | 22 | 21 | 23 :
            values = getCounterValues(layer)
            parser = BuildQualityParser("ctr")
        case _:
            return []

    point_flags = parser(layer)

    points = []
    for idx in range(len(values)):
        points.append(DNP3Point(index=-1, value=values[idx], flags=point_flags[idx]))

    return points