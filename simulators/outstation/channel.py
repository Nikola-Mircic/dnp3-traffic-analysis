from typing import Any

from pydnp3 import opendnp3, asiopal, asiodnp3

LOCAL_IP = "0.0.0.0"
PORT = 20000
_threads_to_allocate = 1

LOG_LEVELS = opendnp3.levels.NORMAL | opendnp3.levels.ALL_COMMS

class ChannelListener(asiodnp3.IChannelListener):
    """
        Override IChannelListener in this manner to implement application-specific channel behavior.
    """
    def __init__(self):
        super(ChannelListener, self).__init__()

    def OnStateChange(self, state):
        print('Channel state change: state={}'.format(state))


class ChannelManager:
    def __init__(self):
        self.log_handler = asiodnp3.ConsoleLogger().Create()
        self.manager = asiodnp3.DNP3Manager(_threads_to_allocate)

        self.retry_parameters = asiopal.ChannelRetry().Default()
        self.listener = ChannelListener()

        self.channel = self.manager.AddTCPServer("Outstation Server",
                                     LOG_LEVELS,
                                     self.retry_parameters,
                                     LOCAL_IP,
                                     PORT,
                                     self.listener)

    def Shutdown(self):
        self.manager.Shutdown()

    def AddOutstation(self,
                      id: Any,
                      commandHandler: Any,
                      application: Any,
                      config: Any):
        return self.channel.AddOutstation(id, commandHandler, application, config)