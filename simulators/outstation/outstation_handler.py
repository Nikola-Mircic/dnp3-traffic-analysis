from pydnp3 import opendnp3, openpal, asiopal, asiodnp3
from pydnp3.opendnp3 import OperateType, ControlRelayOutputBlock

from command_handler import OutstationCommandHandler
import logging
import sys

stdout_stream = logging.StreamHandler(sys.stdout)
stdout_stream.setFormatter(logging.Formatter('%(asctime)s\t%(name)s\t%(levelname)s\t%(message)s'))

_log = logging.getLogger(__name__)
_log.addHandler(stdout_stream)
_log.setLevel(logging.DEBUG)

class OutstationHandler(opendnp3.IOutstationApplication):
    def __init__(self, outstation_name, channel, config):
        super().__init__()
        self._outstation_ptr = None
        self.channel = channel
        self.outstation_name = outstation_name

        self.cmd_handler = OutstationCommandHandler(self)

        self._outstation_ptr = self.channel.AddOutstation(
            self.outstation_name,
            self.cmd_handler,
            self,
            config)

        self._outstation_ptr.Enable()

    def shutdown(self):
        self.channel.Shutdown()

    # Overridden method
    def ColdRestartSupport(self):
        """Return a RestartMode enumerated value indicating whether cold restart is supported."""
        _log.debug('Checking cold restart support...')
        return opendnp3.RestartMode.UNSUPPORTED

    # Overridden method
    def GetApplicationIIN(self):
        """Return the application-controlled IIN field."""
        application_iin = opendnp3.ApplicationIIN()
        application_iin.configCorrupt = False
        application_iin.deviceTrouble = False
        application_iin.localControl = False
        application_iin.needTime = False
        # Just for testing purposes, convert it to an IINField and display the contents of the two bytes.
        iin_field = application_iin.ToIIN()
        _log.debug('Outstation IIN flags: IINField LSB={}, MSB={}'.format(iin_field.LSB, iin_field.MSB))

        return application_iin

    # Overridden method
    def SupportsAssignClass(self):
        _log.debug('Checking class assigning support...')
        return False

    # Overridden method
    def SupportsWriteAbsoluteTime(self):
        _log.debug('Checking write absolute time support...')
        return False

    # Overridden method
    def SupportsWriteTimeAndInterval(self):
        _log.debug('Checking write time interval support...')
        return False

    # Overridden method
    def WarmRestartSupport(self):
        """Return a RestartMode enumerated value indicating whether a warm restart is supported."""
        _log.debug('Checking warm restart support...')
        return opendnp3.RestartMode.UNSUPPORTED

    def process_point_value(self, command_type, command, index, op_type):
        """
            A PointValue was received from the Master. Process its payload.

        :param command_type: (string) Either 'Select' or 'Operate'.
        :param command: A ControlRelayOutputBlock or else a wrapped data value (AnalogOutputInt16, etc.).
        :param index: (integer) DNP3 index of the payload's data definition.
        :param op_type: An OperateType, or None if command_type == 'Select'.
        """
        _log.debug('Processing received point value for index {}: {}'.format(index, command))

    def update(self, value, index):
        """
            Record an opendnp3 data value (Analog, Binary, etc.) in the outstation's database.

            The data value gets sent to the Master as a side-effect.

        :param value: An instance of Analog, Binary, or another opendnp3 data value.
        :param index: (integer) Index of the data definition in the opendnp3 database.
        """
        _log.debug('Recording {} measurement, index={}, value={}'.format(type(value).__name__, index, value.value))
        builder = asiodnp3.UpdateBuilder()
        builder.Update(value, index)
        update = builder.Build()
        self._outstation_ptr.Apply(update)