from pydnp3 import opendnp3, asiodnp3
import threading

from command_handler import OutstationCommandHandler

class OutstationHandler(opendnp3.IOutstationApplication):
    def __init__(self, outstation_name, channel, config):
        super(OutstationHandler, self).__init__()
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
    def GetApplicationIIN(self):
        """Return the application-controlled IIN field."""
        application_iin = opendnp3.ApplicationIIN()
        application_iin.configCorrupt = False
        application_iin.deviceTrouble = False
        application_iin.localControl = False
        application_iin.needTime = False

        return application_iin

    # Overridden method
    def ColdRestartSupport(self):
        """Return a RestartMode enumerated value indicating whether cold restart is supported."""
        print('Checking cold restart support...')
        return opendnp3.RestartMode.SUPPORTED_DELAY_COARSE

    # Overridden method
    def WarmRestartSupport(self):
        """Return a RestartMode enumerated value indicating whether a warm restart is supported."""
        print('Checking warm restart support...')
        return opendnp3.RestartMode.SUPPORTED_DELAY_FINE

    def ColdRestart(self):
        print('Restarting [ COLD ]....')
        return 5 # COARSE --> time in seconds

    def WarmRestart(self):
        print('Restarting [ WARM ]....')
        return 4500 # FINE --> time in milliseconds

    # Overridden method
    def SupportsAssignClass(self):
        print('Checking class assigning support...')
        return False

    # Overridden method
    def SupportsWriteAbsoluteTime(self):
        print('Checking write absolute time support...')
        return True

    # Overridden method
    def SupportsWriteTimeAndInterval(self):
        print('Checking write time interval support...')
        return False


    def process_point_value(self, command_type, command, index, op_type):
        """
        A command was received from the Master. Validate it on Select,
        and actually apply it to the database on Operate - mimicking how
        a real device validates a command before actuating hardware, then
        reports the resulting output state back to the Master.

        :param command_type: (string) Either 'Select' or 'Operate'.
        :param command: A ControlRelayOutputBlock or else a wrapped data value (AnalogOutputInt16, etc.).
        :param index: (integer) DNP3 index of the payload's data definition.
        :param op_type: An OperateType, or None if command_type == 'Select'.
        """
        print('Processing {} for index {}: {}'.format(command_type, index, command))

        # Select is a feasibility check only - a real device would confirm the
        # point exists and the command is well-formed, but must NOT actuate
        # anything yet. Only Operate should cause a real state change.
        if command_type == 'Select':
            return True

        if isinstance(command, opendnp3.ControlRelayOutputBlock):
            # Execute the CROB command
            return self._apply_binary_output_command(command, index)
        elif isinstance(command, (opendnp3.AnalogOutputInt16,
                                   opendnp3.AnalogOutputInt32,
                                   opendnp3.AnalogOutputFloat32,
                                   opendnp3.AnalogOutputDouble64)):
            return  self._apply_analog_output_command(command, index)
        else:
            print('Unrecognized command type for index {}: {}'.format(index, type(command)))
            return False

    def _apply_binary_output_command(self, command, index):
        """
        Actuate a CROB (Group 12) command and reflect the result in
        Binary Output Status (Group 10), the way a real relay/breaker
        would report its new state after actuation.

        - LATCH_ON / LATCH_OFF: maintained state change (e.g. a simple
          relay or maintained contact) - stays until commanded otherwise.
        - CLOSE_PULSE_ON / TRIP_PULSE_ON: two-output breaker control -
          the close/trip coil pulses for onTimeMS, but the resulting
          breaker position (closed/open) persists after the pulse ends.
        - PULSE_ON: activation-model pulse (e.g. a horn or light) -
          output goes active for onTimeMS then automatically reverts.
        - PULSE_OFF: non-interoperable - should not be used for newer apps
        """
        code = command.functionCode

        print("Executing {} for index {}: {}".format(code, index, command))

        on_codes = (opendnp3.ControlCode.LATCH_ON,
                    opendnp3.ControlCode.LATCH_ON_CANCEL)

        off_codes = (opendnp3.ControlCode.LATCH_OFF,
                     opendnp3.ControlCode.LATCH_OFF_CANCEL)

        on_pulse = (opendnp3.ControlCode.PULSE_ON,
                    opendnp3.ControlCode.CLOSE_PULSE_ON,
                    opendnp3.ControlCode.CLOSE_PULSE_ON_CANCEL)

        off_pulse = (opendnp3.ControlCode.TRIP_PULSE_ON,
                     opendnp3.ControlCode.TRIP_PULSE_ON_CANCEL)

        if code in on_codes:
            print('Binary output index {} -> ON (maintained)'.format(index))
            self.update(opendnp3.BinaryOutputStatus(True), index)
            return True

        elif code in off_codes:
            print('Binary output index {} -> OFF (maintained)'.format(index))
            self.update(opendnp3.BinaryOutputStatus(False), index)
            return True

        elif code in on_pulse:
            print('Binary output index {} -> ON (pulse, {}ms)'.format(index, command.onTimeMS))
            self.update(opendnp3.BinaryOutputStatus(True), index)

            on_time_seconds = max(command.onTimeMS, 0) / 1000.0
            timer = threading.Timer(
                on_time_seconds,
                lambda: self.update(opendnp3.BinaryOutputStatus(False), index)
            )
            timer.daemon = True
            timer.start()

            return True
        elif code in off_pulse:
            print('Binary output index {} -> OFF (pulse, {}ms)'.format(index, command.onTimeMS))
            self.update(opendnp3.BinaryOutputStatus(False), index)

            off_time = max(command.onTimeMS, 0) / 1000.0
            timer = threading.Timer(
                off_time,
                lambda: self.update(opendnp3.BinaryOutputStatus(False), index)
            )
            timer.daemon = True
            timer.start()

            return True
        else:
            print('Unsupported/undefined control code {} for index {}'.format(code, index))
            return False

    def _apply_analog_output_command(self, command, index):
        """
        Actuate an Analog Output (Group 41) command and reflect the
        commanded value in Analog Output Status (Group 40), the way a
        real analog output card would report the value it's now driving.
        """
        print('Analog output index {} -> {}'.format(index, command.value))
        self.update(opendnp3.AnalogOutputStatus(command.value), index)
        return True

    def update(self, value, index):
        """
        Record an opendnp3 data value (Analog, Binary, etc.) in the outstation's database.

        The data value gets sent to the Master as a side-effect.

        :param value: An instance of Analog, Binary, or another opendnp3 data value.
        :param index: (integer) Index of the data definition in the opendnp3 database.
        """
        builder = asiodnp3.UpdateBuilder()
        builder.Update(value, index)
        update = builder.Build()
        self._outstation_ptr.Apply(update)