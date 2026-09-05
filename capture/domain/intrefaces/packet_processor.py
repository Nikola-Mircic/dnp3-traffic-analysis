from abc import ABC, abstractmethod
from capture.domain.models.dnp3_packet import DNP3Frame


class PacketProcessor(ABC):
    """Consumes DNP3Frame objects. Only ever sees the internal model."""

    @abstractmethod
    def process(self, frame: DNP3Frame) -> None:
        raise NotImplementedError