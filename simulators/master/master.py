from pydnp3 import opendnp3, asiodnp3, openpal

from soe_handler import SOEHandler

CLASS_0 = 1
CLASS_1 = 2
CLASS_2 = 4
CLASS_3 = 8

class Master(opendnp3.IMasterApplication):
    def __init__(self, master_name, channel, soe_handler = SOEHandler()):
        super(Master, self).__init__()
        self.master_name = master_name
        self.channel = channel
        self.soe_handler = soe_handler

        self.stack_config = asiodnp3.MasterStackConfig()

        self.stack_config.master.disableUnsolOnStartup = False
        self.stack_config.master.ignoreRestartIIN = False
        self.stack_config.master.integrityOnEventOverflowIIN = False

        self.stack_config.link.LocalAddr = 2
        self.stack_config.link.RemoteAddr = 1
        self.stack_config.link.IsMaster = True


        self._master_ptr = self.channel.AddMaster(self.master_name,
                                                  self.soe_handler,
                                                  self,
                                                  self.stack_config)

        self.scan = self._master_ptr.AddClassScan(opendnp3.ClassField().AllClasses(),
                                                     openpal.TimeDuration().Seconds(2),
                                                     opendnp3.TaskConfig().Default())

        # self.scan_c1 = self._master_ptr.AddClassScan(opendnp3.ClassField(CLASS_1),
        #                                           openpal.TimeDuration().Seconds(2),
        #                                           opendnp3.TaskConfig().Default())
        #
        # self.scan_c2 = self._master_ptr.AddClassScan(opendnp3.ClassField(CLASS_2),
        #                                           openpal.TimeDuration().Seconds(5),
        #                                           opendnp3.TaskConfig().Default())
        #
        # self.scan_c3 = self._master_ptr.AddClassScan(opendnp3.ClassField(CLASS_3),
        #                                           openpal.TimeDuration().Seconds(10),
        #                                           opendnp3.TaskConfig().Default())
        #
        # self.scan_integrity = self._master_ptr.AddClassScan(opendnp3.ClassField(CLASS_0 | CLASS_1 | CLASS_2 | CLASS_3),
        #                                              openpal.TimeDuration().Seconds(60),
        #                                              opendnp3.TaskConfig().Default())

        self._master_ptr.Enable()


    def Shutdown(self):
        self._master_ptr.Shutdown()
        self.channel.Shutdown()

    def AssignClassDuringStartup(self):
        return False

