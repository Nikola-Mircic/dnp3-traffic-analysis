from dnp3_python.dnp3station import outstation
from pydnp3 import opendnp3, asiodnp3, openpal
from pydnp3.opendnp3 import (BinaryConfig,
                             AnalogConfig,
                             CounterConfig,
                             FrozenCounterConfig,
                             BOStatusConfig,
                             AOStatusConfig)

from command_handler import OutstationCommandHandler
from outstation_handler import OutstationHandler

class OutstationConfigBuilder():
    def __init__(self):
        self.local_addr=2
        self.remote_addr=1
        self.binary_inputs = []
        self.analog_inputs = []
        self.counters = []
        self.frozen_counters = []

        self.binary_outputs = []
        self.analog_outputs = []
        self.allow_unsolicited = False
        self.event_buffer_size = 10

    def _configure_stack(self):
        database_sizes = opendnp3.DatabaseSizes(
            len(self.binary_inputs),
            0,
            len(self.analog_inputs),
            len(self.counters),
            len(self.frozen_counters),
            len(self.binary_outputs),
            len(self.analog_outputs),
            0
        )

        stack = asiodnp3.OutstationStackConfig(database_sizes)

        # Set same buffer size for all event types
        stack.outstation.eventBufferConfig = opendnp3.EventBufferConfig(
            maxBinaryEvents=self.event_buffer_size,
            maxAnalogEvents=self.event_buffer_size,
            maxCounterEvents=self.event_buffer_size,
            maxFrozenCounterEvents=self.event_buffer_size,
        )

        stack.outstation.params.allowUnsolicited = self.allow_unsolicited

        stack.link.LocalAddr = self.local_addr
        stack.link.RemoteAddr = self.remote_addr
        stack.link.KeepAliveTimeout = openpal.TimeDuration().Max()

        return stack

    def _apply_config(self, dest, src):
        for i in range(len(src)):
            dest[i].clazz = src[i].clazz
            dest[i].svariation = src[i].svariation
            dest[i].evariation = src[i].evariation

    def _configure_database(self, dbConfig):
        """ Configure object groups for points """

        self._apply_config(dbConfig.binary, self.binary_inputs)
        self._apply_config(dbConfig.analog, self.analog_inputs)
        self._apply_config(dbConfig.counter, self.counters)
        self._apply_config(dbConfig.frozenCounter, self.frozen_counters)
        self._apply_config(dbConfig.boStatus, self.binary_outputs)
        self._apply_config(dbConfig.aoStatus, self.analog_outputs)

    def build(self):
        # Configure outstation stack
        stack = self._configure_stack()

        # Configure point database
        self._configure_database(stack.dbConfig)

        return stack

    def set_local_addr(self, local_addr):
        self.local_addr = local_addr

    def set_remote_addr(self, remote_addr):
        self.remote_addr = remote_addr

    def add_binary_inputs(self, 
                          binary_input_count,
                          cls,
                          svariaton = opendnp3.StaticBinaryVariation.Group1Var2,
                          evariation = opendnp3.EventBinaryVariation.Group2Var2,):
        """Adds binary_input_count input point of class cls to outstation database.

        Uses group 1 and variation 2 ( static input with flags ) and group 2 variation 2
        ( input event with flags and absolute time ) for object types by default. by default.
        """
        for i in range(binary_input_count):
            config = BinaryConfig()
            config.clazz = cls
            config.svariation = svariaton
            config.evariation = evariation
            self.binary_inputs.append(config)


    def add_analog_inputs(self,
                          analog_input_count,
                          cls,
                          svariaton = opendnp3.StaticAnalogVariation.Group30Var5,
                          evariation = opendnp3.EventAnalogVariation.Group32Var7,):
        """Adds analog_input_count input point of class cls to outstation database.

        Uses group 30 and variation 5 ( static 32-bit floating point value with flag )
        and group 32 and variation 7 ( 32-bit floating point event value with flag and
        event time ) for object types by default.
        """
        for i in range(analog_input_count):
            config = AnalogConfig()
            config.clazz = cls
            config.svariation = svariaton
            config.evariation = evariation
            self.analog_inputs.append(config)


    def add_counters(self,
                     counter_count,
                     cls,
                     svariaton = opendnp3.StaticCounterVariation.Group20Var1,
                     evariation = opendnp3.EventCounterVariation.Group22Var1):
        """ Adds counter_count input point of class cls to outstation database.

        Uses group 20 and variation 1 ( static 32-bit with flags ) and group 22 and variation 1
        ( 32-bit event data with flags ) for object types by default.
        """
        for i in range(counter_count):
            config = CounterConfig()
            config.clazz = cls
            config.svariation = svariaton
            config.evariation = evariation
            self.counters.append(config)


    def add_frozen_counters(self,
                            frozen_counter_count,
                            cls,
                            svariation = opendnp3.StaticFrozenCounterVariation.Group21Var5,
                            evariation = opendnp3.EventFrozenCounterVariation.Group23Var5,):
        """ Adds frozen_counter_count input point of class cls to outstation database.

        Uses group 21 and variation 5 ( static 32-bit with flag and time ) and group 23 and
        variation 5 ( 32-bit event with flag and time ) for object types by default.
        """
        for i in range(frozen_counter_count):
            config = FrozenCounterConfig()
            config.clazz = cls
            config.svariation = svariation
            config.evariation = evariation
            self.frozen_counters.append(config)


    def add_binary_outputs(self,
                           binary_output_count,
                           cls,
                           svariaton = opendnp3.StaticBinaryOutputStatusVariation.Group10Var2,
                           evariation = opendnp3.EventBinaryOutputStatusVariation.Group11Var2):
        """Adds binary_output_count input point of class cls to outstation database.

        Uses group 10 and variation 1 ( output status with flags ) and group 11
        and variation 1 ( status with flags ) for object types by default.
        """
        for i in range(binary_output_count):
            config = BOStatusConfig()
            config.clazz = cls
            config.svariation = svariaton
            config.evariation = evariation
            self.binary_outputs.append(config)


    def add_analog_outputs(self,
                            analog_output_count,
                            cls,
                            svariaton = opendnp3.StaticAnalogOutputStatusVariation.Group40Var1,
                            evariation = opendnp3.EventAnalogOutputStatusVariation.Group42Var1):
        """Adds analog_output_count input point of class cls to outstation database.

        Uses group 40 and variation 1 ( output status with flags ) and group 42
        and variation 1 ( output status with flags ) for object types by default."""
        for i in range(analog_output_count):
            config = AOStatusConfig()
            config.clazz = cls
            config.svariation = svariaton
            config.evariation = evariation
            self.analog_outputs.append(config)


    def allow_unsolicited(self):
        self.allow_unsolicited = True


    def disable_unsolicited(self):
        self.allow_unsolicited = False


    def set_event_buffer_size(self, event_buffer_size):
        self.event_buffer_size = event_buffer_size


