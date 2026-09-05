from abc import ABC, abstractmethod
from typing import Iterator, Any


class PacketSource(ABC):
    """Produces raw packet objects. No DNP3 awareness required."""

    @abstractmethod
    def packets(self) -> Iterator[Any]:
        """Yield raw captured packet objects (e.g. pyshark Packet)."""
        raise NotImplementedError