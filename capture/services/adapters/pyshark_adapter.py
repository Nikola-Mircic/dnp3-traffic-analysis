from pyshark.packet.layers.xml_layer import XmlLayer

from domain.intrefaces.packet_adapter import PacketAdapter
from domain.models.dnp3_packet import FunctionCode, DataLinkHeader, ControlField, TransportHeader, ApplicationHeader, \
    IINFlags, DNP3Point, DNP3Object, DNP3Frame, Qualifier
from services.adapters.pyshark_point_parsers import ParsePoints


def _get_raw(layer, field, default: str):
    return str(getattr(layer, field, default))


def _get_int(layer, field, default=0):
    try:
        return int(_get_raw(layer, field, "0"))
    except ValueError:
        return default

def _get_bool(layer, field):
    return _get_raw(layer, field, "False").lower() == "true"

def _get_qualifier(prefix_val, range_val):
    value = (prefix_val & 0x0F) << 4 | range_val & 0x0F
    return Qualifier(value)

def _parse_group_and_var(obj_field):
    gv_value = int(obj_field.get_default_value(), 16)
    group = gv_value >> 8 & 0xFF
    var = gv_value & 0xFF

    return group, var

class PySharkAdapter(PacketAdapter):

    def adapt(self, raw_packet):
        if not hasattr(raw_packet, "dnp3"):
            return None

        dnp3_layer = raw_packet.dnp3

        return DNP3Frame(
            captured_at= self._extract_timestamp(raw_packet),
            data_link=self._parse_data_link(dnp3_layer),
            transport=self._parse_transport(dnp3_layer),
            application=self._parse_application(dnp3_layer),
            objects=self._parse_objects(dnp3_layer)
        )

    @staticmethod
    def _extract_timestamp(raw_packet):
        return raw_packet.sniff_time

    @staticmethod
    def _parse_data_link(layer):

        source = _get_int(layer, "src")
        destination = _get_int(layer, "dst")

        control = ControlField(
            dir=_get_bool(layer, "ctl_dir"),
            primary=_get_bool(layer, "ctl_prm"),
            fcb=_get_bool(layer, "ctl_fcb"),
            fcv=_get_bool(layer, "ctl_fcv"),
            function_code=_get_int(layer, "ctl_func"),
        )

        return DataLinkHeader(
            source=source,
            destination=destination,
            frame_length=_get_int(layer, "len"),
            control=control,
        )

    @staticmethod
    def _parse_transport(layer):
        sequence = _get_int(layer, "tr_seq")

        return TransportHeader(
            first=_get_bool(layer, "tr_fir"),
            final=_get_bool(layer, "tr_fin"),
            sequence=sequence,
        )

    @staticmethod
    def _parse_application(layer):
        func_raw = _get_int(layer, "al_func", default=FunctionCode.UNDEFINED)

        try:
            function_code = FunctionCode(func_raw)
        except ValueError:
            function_code = func_raw  # unknown/vendor-specific code, keep raw int

        iin = None
        if function_code in (FunctionCode.RESPONSE, FunctionCode.UNSOLICITED_RESPONSE):
            iin = IINFlags(
                broadcast=_get_bool(layer, "al_iin_bmsg"),
                class_1_events=_get_bool(layer, "al_iin_cls1d"),
                class_2_events=_get_bool(layer, "al_iin_cls2d"),
                class_3_events=_get_bool(layer, "al_iin_cls3d"),
                need_time=_get_bool(layer, "al_iin_tsr"),
                local_control=_get_bool(layer, "al_iin_dol"),
                device_trouble=_get_bool(layer, "al_iin_dt"),
                device_restart=_get_bool(layer, "al_iin_rst"),
                no_func_code_support=_get_bool(layer, "al_iin_fcni"),
                object_unknown=_get_bool(layer, "al_iin_obju"),
                parameter_error=_get_bool(layer, "al_iin_pioor"),
                event_buffer_overflow=_get_bool(layer, "al_iin_ebo"),
                already_executing=_get_bool(layer, "al_iin_oae"),
                config_corrupt=_get_bool(layer, "al_iin_cc"),
            )

        return ApplicationHeader(
            first=_get_bool(layer, "al_fir"),
            final=_get_bool(layer, "al_fin"),
            confirm_requested=_get_bool(layer, "al_con"),
            unsolicited=_get_bool(layer, "al_uns"),
            sequence=_get_int(layer, "al_seq"),
            function_code=FunctionCode(function_code),
            iin=iin,
        )

    @staticmethod
    def _parse_objects(layer: XmlLayer):
        """
            'al_fragments', 'al_fragment', 'al_fragment_count', 'al_fragment_reassembled_length',
             'al_obj', 'al_objq_prefix', 'al_objq_range', 'al_range_quantity', 'al_index',
             'al_biq_b7', 'al_biq_b6', 'al_biq_b5', 'al_biq_b4', 'al_biq_b3', 'al_biq_b2', 'al_biq_b1', 'al_biq_b0',
             'al_aiq_b7', 'al_aiq_b6', 'al_aiq_b5', 'al_aiq_b4', 'al_aiq_b3', 'al_aiq_b2', 'al_aiq_b1', 'al_aiq_b0',
             'al_ana_int',
             'al_ctrq_b7', 'al_ctrq_b6', 'al_ctrq_b5', 'al_ctrq_b4', 'al_ctrq_b3', 'al_ctrq_b2', 'al_ctrq_b1', 'al_ctrq_b0',
             'al_cnt'"""
        try:
            if not hasattr(layer, "al_obj"):
                return []
            # 'al_fragments', 'al_fragment', 'al_fragment_count', 'al_fragment_reassembled_length'
            print(layer.field_names)
            # List of groups and variations for each object
            gv_list = list(map(lambda x: _parse_group_and_var(x), layer.get_field("al_obj").all_fields))
            # Print groups and variations
            print("Object gv: {}".format(list(map(lambda gv: str(gv[0])+"."+str(gv[1]), gv_list))))
            prefix_values = map(lambda x: x.int_value, layer.get_field("al_objq_prefix").all_fields)
            range_values = map(lambda x: x.int_value, layer.get_field("al_objq_range").all_fields)
            qualifier_values = list(zip(prefix_values, range_values))
            qualifiers = [_get_qualifier(x,y) for x,y in qualifier_values]
            point_counts = list(map(lambda x: int(x.get_default_value()), layer.get_field("al_range_quantity").all_fields))
            index_values = list(map(lambda x: int(x.get_default_value()), layer.get_field("al_index").all_fields))
            print("Qualifiers: {}".format(qualifiers))
            print("Points per object: {}".format(point_counts))
            print("Index values: {}".format(index_values))

            objects = []

            for object_idx in range(len(gv_list)):
                group = gv_list[object_idx][0]
                variation = gv_list[object_idx][1]
                qualifier = qualifiers[object_idx]
                points = ParsePoints(group, variation, layer)
                objects.append(DNP3Object(group=group, variation=variation, qualifier=qualifier, points=points))

            return objects

        except Exception as e:
            print(e)

        return []