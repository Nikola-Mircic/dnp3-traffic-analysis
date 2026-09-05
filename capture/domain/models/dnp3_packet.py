from datetime import datetime
from dataclasses import dataclass, field
from enum import IntEnum
from typing import Optional, Any


class FunctionCode(IntEnum):
    # Control
    CONFIRM = 0x00 # Used when master confirm reception with the CON flag set
    READ = 0x01 # Request the input data
    WRITE = 0x02 # Clear outstation's 'restart' flag or write the time-of-day
    SELECT = 0x03 # Control output points using two-step select-before-operate sequence
    OPERATE = 0x04 # Control output points using two-step select-before-operate sequence
    DIRECT_OPERATE = 0x05 # Control output points with one message
    DIRECT_OPERATE_NR = 0x06 # Direct operate without response

    # Freeze
    FREEZE = 0x07 # Freeze counters
    FREEZE_NR = 0x08 # Freeze counter without response
    FREEZE_CLEAR = 0x09 # Freeze counter and set running counter value to 0
    FREEZE_CLEAR_NR = 0x0A # Freeze counter and set running counter value to 0 without response

    # Restart
    COLD_RESTART = 0x0D # Initialize cold reboot on outstation
    WARM_RESTART = 0x0E # Initialize partial restart

    # Configuration
    ENABLE_UNSOLICITED = 0x14 # Allow outstation to send unsolicited responses for specified classes
    DISABLE_UNSOLICITED = 0x15 # Stop outstation from sending unsolicited responses
    ASSIGN_CLASS = 0x16 # Master assign classes to data points

    # Response
    RESPONSE = 0x81 # Standard reply
    UNSOLICITED_RESPONSE = 0x82 # Outstation-initiated messages sent without preceding request


@dataclass
class IINFlags:
    # First IIN [lower octet ]
    broadcast: bool = False # Outstation received broadcast message
    class_1_events: bool = False # There is at least on class 1 event not sent yet to the master
    class_2_events: bool = False # There is at least on class 2 event not sent yet to the master
    class_3_events: bool = False # There is at least on class 3 event not sent yet to the master
    need_time: bool = False # Device needs time synchronization
    local_control: bool = False # One or more output points are in 'local' mode ( will not receive operation commands )
    device_trouble: bool = False # There is a problem at outstation
    device_restart: bool = False # Outstation has restarted
    no_func_code_support: bool = False # Error IIN for not supported function code
    object_unknown: bool = False # Error IIN for unknown object from request
    parameter_error: bool = False # Error IIN for other errors in most recent request
    event_buffer_overflow: bool = False # At least one event is lost due to outstation running out of memory
    already_executing: bool = False # Operation can't be processed because operation from one of previous requests is not completed yet
    config_corrupt: bool = False # There is an error at outstations configuration - can't continue


@dataclass
class ControlField:
    """Class representing a control octet from data link header."""
    dir: bool
    primary: bool
    fcb: bool
    fcv: bool
    function_code: int


@dataclass
class DataLinkHeader:
    """ Class representing a data link header.

    A data link header consists of 10 octets:
        - 0x05             - *Default data link header byte* [ 1 byte ],
        - 0x64             - *Default data link header byte* [ 1 byte ],
        - **frame_length** - Length of the frame [ 1 byte ],
        - **control**      - Control flags [1 byte],
        - **destination**  - Destination address 0-0xFFEF [ 2 bytes ],
        - **source**       - Source address 0-0xFFEF [ 2 bytes ],
        - crc              - CRC [ 2 bytes ],
    """
    source: int
    destination: int
    frame_length: int
    control: ControlField


@dataclass
class TransportHeader:
    """Class representing a transport header.

    Single byte segment containing 3 values:
        - **first**     - Set for the first frame in a fragment [ 1 bit ],
        - **final**     - Set for the last frame in a fragment [ 1 bit ],
        - **sequence**  - Value containing a sequence number of the frame [ 6 bits ],
    """
    first: bool   # FIR
    final: bool   # FIN
    sequence: int

#A single data point within an object (analog in, binary out)
@dataclass
class DNP3Point:
    index: int
    value: Any
    flags: Optional[int] = None
    timestamp: Optional[datetime] = None


@dataclass
class DNP3Object:
    """Class representing a DNP3 object.
    It has three parts:
        - **group** - Determines what is being sent [ 1 byte ]
        - **variation** - Determines the item format [ 1 byte ]
        - **qualifier** - How items are organized [ 1 byte ]
        - **points** - List of points within an object
    """
    group: int
    variation: int
    qualifier: int
    points: list[DNP3Point] = field(default_factory=list)


@dataclass
class ApplicationHeader:
    """Class representing an application header.

    An application header is a 16-bit sequence ( 32-bit sequence if the message is coming from an outstation ).
    It contains next values:
        - **first** - Set for the first application fragment [ 1 bit ],
        - **final** - Set for the last application fragment [ 1 bit ],
        - **confirm_requested** - Set if message request confirmation upon arrival. Set only on messages sent
                                    from an outstation. [ 1 bit ]
        - **unsolicited** - Set for unsolicited messages. [ 1 bit ]
        - **sequence** - Value containing a sequence number of the application fragment [ 4 bits ],
    """
    first: bool # FIR
    final: bool # FIN
    confirm_requested: bool # CON
    unsolicited: bool # UNS
    sequence: int
    function_code: FunctionCode
    iin: Optional[IINFlags] = None  # only present on responses


# The unit every component downstream of the Adapter works with.
@dataclass
class DNP3Frame:
    captured_at: datetime
    data_link: DataLinkHeader
    transport: Optional[TransportHeader]
    application: Optional[ApplicationHeader]
    objects: list[DNP3Object] = field(default_factory=list)