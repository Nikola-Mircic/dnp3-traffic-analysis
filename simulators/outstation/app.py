import logging
import sys

import threading
import signal

from pydnp3 import opendnp3

from channel import ChannelManager
from config_builder import OutstationConfigBuilder
from outstation_handler import OutstationHandler
from point_simulator import PointSimulator, CsvAnalogInputFunction, CsvCountersFunction

stdout_stream = logging.StreamHandler(sys.stdout)
stdout_stream.setFormatter(logging.Formatter('%(asctime)s\t%(name)s\t%(levelname)s\t%(message)s'))

_log = logging.getLogger(__name__)
_log.addHandler(stdout_stream)
_log.setLevel(logging.DEBUG)

csv_filepath = "turbines.csv"

def main():
    """The Outstation has been started from the command line. Keep the process alive to serve requests."""
    channel = ChannelManager()

    # Generating outstation config
    builder = OutstationConfigBuilder()
    builder.set_event_buffer_size(5)
    builder.set_local_addr(2)
    builder.set_remote_addr(1)
    # 3 analog inputs for 3 phases ( voltage )
    builder.add_analog_inputs(3, opendnp3.PointClass.Class1)
    # 1 counter for total energy exported
    builder.add_counters(1, opendnp3.PointClass.Class1)
    config = builder.build()

    app = OutstationHandler("New outstation", channel, config)

    simulator = PointSimulator(app, 5)

    simulator.add_analog_input_function(0, CsvAnalogInputFunction(csv_filepath, "Voltage L1 / U (V)"))
    simulator.add_analog_input_function(1, CsvAnalogInputFunction(csv_filepath, "Voltage L2 / V (V)"))
    simulator.add_analog_input_function(2, CsvAnalogInputFunction(csv_filepath, "Voltage L3 / W (V)"))

    simulator.add_counters_function(0, CsvCountersFunction(csv_filepath, "Energy Export counter (kWh)"))

    simulator.start()

    stop = threading.Event()
    signal.signal(signal.SIGINT, lambda *_: stop.set())
    signal.signal(signal.SIGTERM, lambda *_: stop.set())

    print("Running. Press Ctrl+C or stop the container to exit.")
    stop.wait()

    _log.debug("Exiting...")
    simulator.stop()
    app.shutdown()

if __name__ == '__main__':
    main()