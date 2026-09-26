from domain.intrefaces.packet_adapter import PacketAdapter
from domain.models.dnp3_packet import FunctionCode, DataLinkHeader, ControlField, TransportHeader, ApplicationHeader, IINFlags, DNP3Point, DNP3Object, DNP3Frame

def _get_raw(layer, field, default: str):
    return str(getattr(layer, field, default))


def _get_int(layer, field, default=0):
    try:
        return int(_get_raw(layer, field, "0"))
    except ValueError:
        return default

def _get_bool(layer, field):
    return _get_raw(layer, field, "False").lower() == "true"

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
            objects=[]
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