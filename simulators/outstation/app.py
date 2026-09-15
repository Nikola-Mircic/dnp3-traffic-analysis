import logging
import random
import signal
import sys
import threading
import time

from pydnp3 import opendnp3

from channel.channel import ChannelManager
from config_builder import OutstationConfigBuilder
from outstation_handler import OutstationHandler

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
                self.app.update(opendnp3.Binary(self._binary_state[index]), index)

            # Analog Input: random measured value.
            for index in ANALOG_INDEXES:
                value = round(random.uniform(0.0, 100.0), 3)
                self.app.update(opendnp3.Analog(value), index)

            # Counter: monotonically increasing count.
            for index in COUNTER_INDEXES:
                self._counter_value[index] += random.randint(1, 5)
                self.app.update(opendnp3.Counter(self._counter_value[index]), index)

            self._stop_event.wait(self.interval)
        _log.debug('PointSimulator stopped.')


def main():
    """The Outstation has been started from the command line. Keep the process alive to serve requests."""
    channel = ChannelManager()

    # Generating outstation config
    builder = OutstationConfigBuilder()
    builder.set_event_buffer_size(5)
    builder.set_local_addr(2)
    builder.set_remote_addr(1)
    builder.add_binary_inputs(3, opendnp3.PointClass.Class1)
    builder.add_analog_inputs(2, opendnp3.PointClass.Class2)
    builder.add_counters(2, opendnp3.PointClass.Class3)
    builder.add_binary_outputs(1, opendnp3.PointClass.Class1)
    builder.add_analog_outputs(1, opendnp3.PointClass.Class1)
    config = builder.build()

    app = OutstationHandler("New outstation", channel, config)

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