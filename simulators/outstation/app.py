import logging
import random
import signal
import sys
import threading
import time

from pydnp3 import opendnp3, openpal, asiopal, asiodnp3

LOG_LEVELS = opendnp3.levels.NORMAL | opendnp3.levels.ALL_COMMS
LOCAL_IP = "0.0.0.0"
PORT = 20000

# Indexes used for our six example points. Must line up with configure_database() below.
BINARY_INDEXES = (0, 1)    # Group 1  - Binary Input (digital)
ANALOG_INDEXES = (0, 1)    # Group 30 - Analog Input
COUNTER_INDEXES = (0, 1)   # Group 20 - Counter

# How often the simulator pushes new values into the database, in seconds.
UPDATE_INTERVAL_SECONDS = 5

stdout_stream = logging.StreamHandler(sys.stdout)
stdout_stream.setFormatter(logging.Formatter('%(asctime)s\t%(name)s\t%(levelname)s\t%(message)s'))

_log = logging.getLogger(__name__)
_log.addHandler(stdout_stream)
_log.setLevel(logging.DEBUG)


class OutstationApplication(opendnp3.IOutstationApplication):
    """
        Interface for all outstation callback info except for control requests.

        DNP3 spec section 5.1.6.2:
            The Application Layer provides the following services for the DNP3 User Layer in an outstation:
                - Notifies the DNP3 User Layer when action requests, such as control output,
                  analog output, freeze and file operations, arrive from a master.
                - Requests data and information from the outstation that is wanted by a master
                  and formats the responses returned to a master.
                - Assures that event data is successfully conveyed to a master (using
                  Application Layer confirmation).
                - Sends notifications to the master when the outstation restarts, has queued events,
                  and requires time synchronization.

        DNP3 spec section 5.1.6.3:
            The Application Layer requires specific services from the layers beneath it.
                - Partitioning of fragments into smaller portions for transport reliability.
                - Knowledge of which device(s) were the source of received messages.
                - Transmission of messages to specific devices or to all devices.
                - Message integrity (i.e., error-free reception and transmission of messages).
                - Knowledge of the time when messages arrive.
                - Either precise times of transmission or the ability to set time values
                  into outgoing messages.
    """

    outstation = None

    def __init__(self):
        super(OutstationApplication, self).__init__()

        _log.debug('Configuring the DNP3 stack.')
        self.stack_config = self.configure_stack()

        _log.debug('Configuring the outstation database.')
        self.configure_database(self.stack_config.dbConfig)

        _log.debug('Creating a DNP3Manager.')
        threads_to_allocate = 1
        # self.log_handler = MyLogger()
        self.log_handler = asiodnp3.ConsoleLogger().Create()              # (or use this during regression testing)
        self.manager = asiodnp3.DNP3Manager(threads_to_allocate, self.log_handler)

        _log.debug('Creating the DNP3 channel, a TCP server.')
        self.retry_parameters = asiopal.ChannelRetry().Default()
        self.listener = AppChannelListener()
        # self.listener = asiodnp3.PrintingChannelListener().Create()       # (or use this during regression testing)
        self.channel = self.manager.AddTCPServer("server",
                                                 LOG_LEVELS,
                                                 self.retry_parameters,
                                                 LOCAL_IP,
                                                 PORT,
                                                 self.listener)

        _log.debug('Adding the outstation to the channel.')
        self.command_handler = OutstationCommandHandler()
        # self.command_handler =  opendnp3.SuccessCommandHandler().Create() # (or use this during regression testing)
        self.outstation = self.channel.AddOutstation("outstation", self.command_handler, self, self.stack_config)

        # Put the Outstation singleton in OutstationApplication so that it can be used to send updates to the Master.
        OutstationApplication.set_outstation(self.outstation)

        _log.debug('Enabling the outstation. Traffic will now start to flow.')
        self.outstation.Enable()

    @staticmethod
    def configure_stack():
        """Set up the OpenDNP3 configuration."""
        # Instead of DatabaseSizes.AllTypes(10), which allocates 10 points of
        # EVERY type (70 total, most unused), size the database to exactly the
        # points we configure below. Positional order is:
        # (numBinary, numDoubleBitBinary, numAnalog, numCounter,
        #  numFrozenCounter, numBinaryOutputStatus, numAnalogOutputStatus, reserved)
        database_sizes = opendnp3.DatabaseSizes(
            len(BINARY_INDEXES),   # Binary Input       -> 2
            0,                      # Double-bit Binary  -> unused
            len(ANALOG_INDEXES),   # Analog Input       -> 2
            len(COUNTER_INDEXES),  # Counter            -> 2
            0,                      # Frozen Counter     -> unused
            0,                      # Binary Output Status -> unused
            0,                      # Analog Output Status -> unused
            0                       # reserved
        )
        stack_config = asiodnp3.OutstationStackConfig(database_sizes)
        stack_config.outstation.eventBufferConfig = opendnp3.EventBufferConfig(
            len(BINARY_INDEXES), 0, len(ANALOG_INDEXES), len(COUNTER_INDEXES), 0, 0, 0, 0
        )
        stack_config.outstation.params.allowUnsolicited = True
        stack_config.link.LocalAddr = 2  # Outstation address
        stack_config.link.RemoteAddr = 1  # Master address
        stack_config.link.KeepAliveTimeout = openpal.TimeDuration().Max()
        return stack_config

    @staticmethod
    def configure_database(db_config):
        """
            Configure six example points in the Outstation's database:

            - Binary Input  (Group 1,  digital)  at indexes 0 and 1
            - Analog Input  (Group 30, analog)   at indexes 0 and 1
            - Counter       (Group 20, counter)  at indexes 0 and 1

            The database is now sized to exactly these 6 points (see
            configure_stack), so there are no other leftover/unconfigured
            points sitting in NO_CLASS/RESTART state.
        """
        # --- Binary Input points (digital / on-off status) ---
        for index in BINARY_INDEXES:
            db_config.binary[index].clazz = opendnp3.PointClass.Class1
            db_config.binary[index].svariation = opendnp3.StaticBinaryVariation.Group1Var2
            db_config.binary[index].evariation = opendnp3.EventBinaryVariation.Group2Var2

        # --- Analog Input points (measured value) ---
        for index in ANALOG_INDEXES:
            db_config.analog[index].clazz = opendnp3.PointClass.Class2
            db_config.analog[index].svariation = opendnp3.StaticAnalogVariation.Group30Var1
            db_config.analog[index].evariation = opendnp3.EventAnalogVariation.Group32Var7

        # --- Counter points (accumulating count) ---
        for index in COUNTER_INDEXES:
            db_config.counter[index].clazz = opendnp3.PointClass.Class3
            db_config.counter[index].svariation = opendnp3.StaticCounterVariation.Group20Var1
            db_config.counter[index].evariation = opendnp3.EventCounterVariation.Group22Var1

    def shutdown(self):
        """
            Execute an orderly shutdown of the Outstation.

            The debug messages may be helpful if errors occur during shutdown.
        """
        # _log.debug('Exiting application...')
        # _log.debug('Shutting down outstation...')
        # OutstationApplication.set_outstation(None)
        # _log.debug('Shutting down stack config...')
        # self.stack_config = None
        # _log.debug('Shutting down channel...')
        # self.channel = None
        # _log.debug('Shutting down DNP3Manager...')
        # self.manager = None

        self.manager.Shutdown()

    @classmethod
    def get_outstation(cls):
        """Get the singleton instance of IOutstation."""
        return cls.outstation

    @classmethod
    def set_outstation(cls, outstn):
        """
            Set the singleton instance of IOutstation, as returned from the channel's AddOutstation call.

            Making IOutstation available as a singleton allows other classes (e.g. the command-line UI)
            to send commands to it -- see apply_update().
        """
        cls.outstation = outstn

    # Overridden method
    def ColdRestartSupport(self):
        """Return a RestartMode enumerated value indicating whether cold restart is supported."""
        _log.debug('In OutstationApplication.ColdRestartSupport')
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
        _log.debug('OutstationApplication.GetApplicationIIN: IINField LSB={}, MSB={}'.format(iin_field.LSB,
                                                                                             iin_field.MSB))
        return application_iin

    # Overridden method
    def SupportsAssignClass(self):
        _log.debug('In OutstationApplication.SupportsAssignClass')
        return False

    # Overridden method
    def SupportsWriteAbsoluteTime(self):
        _log.debug('In OutstationApplication.SupportsWriteAbsoluteTime')
        return False

    # Overridden method
    def SupportsWriteTimeAndInterval(self):
        _log.debug('In OutstationApplication.SupportsWriteTimeAndInterval')
        return False

    # Overridden method
    def WarmRestartSupport(self):
        """Return a RestartMode enumerated value indicating whether a warm restart is supported."""
        _log.debug('In OutstationApplication.WarmRestartSupport')
        return opendnp3.RestartMode.UNSUPPORTED

    @classmethod
    def process_point_value(cls, command_type, command, index, op_type):
        """
            A PointValue was received from the Master. Process its payload.

        :param command_type: (string) Either 'Select' or 'Operate'.
        :param command: A ControlRelayOutputBlock or else a wrapped data value (AnalogOutputInt16, etc.).
        :param index: (integer) DNP3 index of the payload's data definition.
        :param op_type: An OperateType, or None if command_type == 'Select'.
        """
        _log.debug('Processing received point value for index {}: {}'.format(index, command))

    def apply_update(self, value, index):
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
        OutstationApplication.get_outstation().Apply(update)


class PointSimulator(threading.Thread):
    """
        Background thread that periodically pushes example values into the six
        configured points (2 Binary, 2 Analog, 2 Counter), so they show live,
        changing data instead of sitting in their initial RESTART state forever.
    """

    def __init__(self, app, interval=UPDATE_INTERVAL_SECONDS):
        super(PointSimulator, self).__init__(daemon=True)
        self.app = app
        self.interval = interval
        self._stop_event = threading.Event()
        self._binary_state = {index: False for index in BINARY_INDEXES}
        self._counter_value = {index: 0 for index in COUNTER_INDEXES}

    def stop(self):
        self._stop_event.set()

    def run(self):
        _log.debug('PointSimulator started, updating points every {}s.'.format(self.interval))
        while not self._stop_event.is_set():
            # Binary Input: toggle each point's state.
            for index in BINARY_INDEXES:
                self._binary_state[index] = not self._binary_state[index]
                self.app.apply_update(opendnp3.Binary(self._binary_state[index]), index)

            # Analog Input: random measured value.
            for index in ANALOG_INDEXES:
                value = round(random.uniform(0.0, 100.0), 3)
                self.app.apply_update(opendnp3.Analog(value), index)

            # Counter: monotonically increasing count.
            for index in COUNTER_INDEXES:
                self._counter_value[index] += random.randint(1, 5)
                self.app.apply_update(opendnp3.Counter(self._counter_value[index]), index)

            self._stop_event.wait(self.interval)
        _log.debug('PointSimulator stopped.')


class OutstationCommandHandler(opendnp3.ICommandHandler):
    """
        Override ICommandHandler in this manner to implement application-specific command handling.

        ICommandHandler implements the Outstation's handling of Select and Operate,
        which relay commands and data from the Master to the Outstation.
    """

    def Start(self):
        _log.debug('In OutstationCommandHandler.Start')

    def End(self):
        _log.debug('In OutstationCommandHandler.End')

    def Select(self, command, index):
        """
            The Master sent a Select command to the Outstation. Handle it.

        :param command: ControlRelayOutputBlock,
                        AnalogOutputInt16, AnalogOutputInt32, AnalogOutputFloat32, or AnalogOutputDouble64.
        :param index: int
        :return: CommandStatus
        """
        OutstationApplication.process_point_value('Select', command, index, None)
        return opendnp3.CommandStatus.SUCCESS

    def Operate(self, command, index, op_type):
        """
            The Master sent an Operate command to the Outstation. Handle it.

        :param command: ControlRelayOutputBlock,
                        AnalogOutputInt16, AnalogOutputInt32, AnalogOutputFloat32, or AnalogOutputDouble64.
        :param index: int
        :param op_type: OperateType
        :return: CommandStatus
        """
        OutstationApplication.process_point_value('Operate', command, index, op_type)
        return opendnp3.CommandStatus.SUCCESS


class AppChannelListener(asiodnp3.IChannelListener):
    """
        Override IChannelListener in this manner to implement application-specific channel behavior.
    """

    def __init__(self):
        super(AppChannelListener, self).__init__()

    def OnStateChange(self, state):
        _log.debug('In AppChannelListener.OnStateChange: state={}'.format(state))


class MyLogger(openpal.ILogHandler):
    """
        Override ILogHandler in this manner to implement application-specific logging behavior.
    """

    def __init__(self):
        super(MyLogger, self).__init__()

    def Log(self, entry):
        filters = entry.filters.GetBitfield()
        location = entry.location.rsplit('/')[-1] if entry.location else ''
        message = entry.message
        _log.debug('Log\tfilters={}\tlocation={}\tentry={}'.format(filters, location, message))


def main():
    """The Outstation has been started from the command line. Keep the process alive to serve requests."""
    app = OutstationApplication()
    _log.debug('Initialization complete. In command loop.')

    simulator = PointSimulator(app)
    simulator.start()

    running = {'value': True}

    def _handle_stop(signum, frame):
        _log.debug('Received signal {}, shutting down.'.format(signum))
        running['value'] = False

    signal.signal(signal.SIGTERM, _handle_stop)
    signal.signal(signal.SIGINT, _handle_stop)

    try:
        # Block here so the Python process (and the container) stays alive
        # and the background DNP3 threads keep serving the TCP listener.
        while running['value']:
            time.sleep(1)
    finally:
        simulator.stop()
        simulator.join(timeout=2)
        app.shutdown()
        _log.debug('Exiting.')
        exit()


if __name__ == '__main__':
    main()