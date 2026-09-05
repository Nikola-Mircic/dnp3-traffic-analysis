from abc import ABC, abstractmethod
from typing import Any, Optional

from capture.domain.models.dnp3_packet import DNP3Frame


class PacketAdapter(ABC):
    """Converts a raw packet into a DNP3Frame, or None if not applicable."""

    @abstractmethod
    def adapt(self, raw_packet: Any) -> Optional[DNP3Frame]:
        raise NotImplementedError