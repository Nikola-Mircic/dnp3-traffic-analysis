import signal
import threading

from master import Master
from channel import ChannelManager

def main():
    master = Master("Master", ChannelManager())
    print('Initialization complete. Polling data.')

    stop = threading.Event()
    signal.signal(signal.SIGINT, lambda *_: stop.set())
    signal.signal(signal.SIGTERM, lambda *_: stop.set())

    stop.wait()
    master.Shutdown()


if __name__ == '__main__':
    main()