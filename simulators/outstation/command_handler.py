from pydnp3 import opendnp3


class OutstationCommandHandler(opendnp3.ICommandHandler):
    def __init__(self, outstation):
        super().__init__()
        self.outstation = outstation

    def Start(self):
        pass

    def End(self):
        pass

    def Select(self, command, index):
        """
        The Master sent a Select command to the Outstation. Handle it.

        :param command: ControlRelayOutputBlock,
                        AnalogOutputInt16, AnalogOutputInt32, AnalogOutputFloat32, or AnalogOutputDouble64.
        :param index: int
        :return: CommandStatus
        """
        if self.outstation.process_point_value('Select', command, index, None):
            return opendnp3.CommandStatus.SUCCESS
        return opendnp3.CommandStatus.CANCELLED

    def Operate(self, command, index, op_type):
        """
        The Master sent an Operate command to the Outstation. Handle it.

        :param command: ControlRelayOutputBlock,
                        AnalogOutputInt16, AnalogOutputInt32, AnalogOutputFloat32, or AnalogOutputDouble64.
        :param index: int
        :param op_type: OperateType
        :return: CommandStatus
        """
        if self.outstation.process_point_value('Operate', command, index, op_type):
            return opendnp3.CommandStatus.SUCCESS

        return opendnp3.CommandStatus.FORMAT_ERROR