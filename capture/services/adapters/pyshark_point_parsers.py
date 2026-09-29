"""
'al_fragments', 'al_fragment', 'al_fragment_count', 'al_fragment_reassembled_length',
 'al_obj', 'al_objq_prefix', 'al_objq_range', 'al_range_quantity', 'al_index',
 'al_biq_b7', 'al_biq_b6', 'al_biq_b5', 'al_biq_b4', 'al_biq_b3', 'al_biq_b2', 'al_biq_b1', 'al_biq_b0',
 'al_aiq_b7', 'al_aiq_b6', 'al_aiq_b5', 'al_aiq_b4', 'al_aiq_b3', 'al_aiq_b2', 'al_aiq_b1', 'al_aiq_b0',
 'al_ana_int',
 'al_ctrq_b7', 'al_ctrq_b6', 'al_ctrq_b5', 'al_ctrq_b4', 'al_ctrq_b3', 'al_ctrq_b2', 'al_ctrq_b1', 'al_ctrq_b0',
 'al_cnt'
 """
import struct
from socket import ntohl
from domain.models.dnp3_packet import PointFlags, DNP3Point

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

def BinaryPointParser(layer):
    """
    Parse values and flags for binary input points from a dnp3 layer of pyshark packet
    :param layer: DNP3 layer from pyshark
    :return: values and flags as two separate lists
    """
    # parse list of quality bits for each point
    values = [f.int_value == 1 for f in layer.get_field("al_biq_b7").all_fields]
    b7_values = [f.int_value == 1 for f in layer.get_field("al_biq_b7").all_fields]
    b6_values = [f.int_value == 1 for f in layer.get_field("al_biq_b6").all_fields]
    b5_values = [f.int_value == 1 for f in layer.get_field("al_biq_b5").all_fields]
    b4_values = [f.int_value == 1 for f in layer.get_field("al_biq_b4").all_fields]
    b3_values = [f.int_value == 1 for f in layer.get_field("al_biq_b3").all_fields]
    b2_values = [f.int_value == 1 for f in layer.get_field("al_biq_b2").all_fields]
    b1_values = [f.int_value == 1 for f in layer.get_field("al_biq_b1").all_fields]
    b0_values = [f.int_value == 1 for f in layer.get_field("al_biq_b0").all_fields]

    qualitiy_bits_values = list(zip(b0_values, b1_values, b2_values, b3_values, b4_values, b5_values, b6_values, b7_values))

    point_flags = [_list_to_quality_obj(bits) for bits in qualitiy_bits_values]

    points = []
    for idx in range(len(values)):
        points.append(DNP3Point(index=idx, value=values[idx], flags=point_flags[idx]))

    return points

def AnalogPointParser(layer):
    values = [int(f.show) for f in layer.get_field("al_ana_int").all_fields]
    b7_values = [f.int_value == 1 for f in layer.get_field("al_aiq_b7").all_fields]
    b6_values = [f.int_value == 1 for f in layer.get_field("al_aiq_b6").all_fields]
    b5_values = [f.int_value == 1 for f in layer.get_field("al_aiq_b5").all_fields]
    b4_values = [f.int_value == 1 for f in layer.get_field("al_aiq_b4").all_fields]
    b3_values = [f.int_value == 1 for f in layer.get_field("al_aiq_b3").all_fields]
    b2_values = [f.int_value == 1 for f in layer.get_field("al_aiq_b2").all_fields]
    b1_values = [f.int_value == 1 for f in layer.get_field("al_aiq_b1").all_fields]
    b0_values = [f.int_value == 1 for f in layer.get_field("al_aiq_b0").all_fields]

    qualitiy_bits_values = list(
        zip(b0_values, b1_values, b2_values, b3_values, b4_values, b5_values, b6_values, b7_values))

    point_flags = [_list_to_quality_obj(bits) for bits in qualitiy_bits_values]

    points = []
    for idx in range(len(values)):
        points.append(DNP3Point(index=idx, value=values[idx], flags=point_flags[idx]))

    return points

def ParsePoints(group, var, layer):
    match group:
        case 1 | 2:
            return BinaryPointParser(layer)
        case 32:
            return AnalogPointParser(layer)
        case _:
            return []