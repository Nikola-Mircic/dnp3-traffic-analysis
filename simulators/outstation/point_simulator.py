import threading
import pandas as pd
from pydnp3 import opendnp3

def CsvBinaryInputFunction(path, colname):
    print("Creating binary input function for {} [{}]".format(colname, path))
    df = pd.read_csv(path)
    def function(app, index, cycles):
        raw = df[colname][cycles+1]
        if not pd.isna(raw):
            value = bool(raw)
            app.update(opendnp3.Binary(value), index)

    return function


def CsvAnalogInputFunction(path, colname):
    print("Creating analog input function for {} [{}]".format(colname, path))
    df = pd.read_csv(path)
    def function(app, index, cycles):
        raw = df[colname][cycles+1]
        if not pd.isna(raw):
            value = float(raw)
            app.update(opendnp3.Analog(value), index)

    return function


def CsvCountersFunction(path, colname):
    print("Creating counter function for {} [{}]".format(colname, path))
    df = pd.read_csv(path)
    def function(app, index, cycles):
        raw = df[colname][cycles+1]
        if not pd.isna(raw):
            value = int(raw)
            app.update(opendnp3.Counter(value), index)

    return function


def CsvFrozenCountersFunction(path, colname):
    print("Creating frozen counter function for {} [{}]".format(colname, path))
    df = pd.read_csv(path)
    def function(app, index, cycles):
        raw = df[colname][cycles+1]
        if not pd.isna(raw):
            value = int(raw)
            app.update(opendnp3.FrozenCounter(value), index)

    return function


class PointSimulator:
    def __init__(self, app, interval):
        self.app = app
        self.interval = interval
        self.cycles = 0
        self.binary_input_functions = []
        self.analog_input_functions = []
        self.counters_functions = []
        self.frozen_counters_functions = []

        self.worker_thread = None
        self.stop_event = threading.Event()

    def start(self):
        self.cycles = 0
        self.worker_thread = threading.Thread(target=self._run_functions)
        self.worker_thread.start()


    def _run_functions(self):
        """ Runs all the functions untile stop_event is set by calling stop() function"""
        while not self.stop_event.wait(self.interval):
            for (index, bfunction) in self.binary_input_functions:
                bfunction(self.app, index, self.cycles)

            for (index, afunction) in self.analog_input_functions:
                afunction(self.app, index, self.cycles)

            for (index, cfunction) in self.counters_functions:
                cfunction(self.app, index, self.cycles)

            for (index, ffunction) in self.frozen_counters_functions:
                ffunction(self.app, index, self.cycles)

            self.cycles += 1

    def stop(self):
        self.stop_event.set()
        if  self.worker_thread is not None:
            self.worker_thread.join()
            self.worker_thread = None

    def add_binary_input_function(self, index, function):
        self.binary_input_functions.append((index, function))

    def add_analog_input_function(self, index, function):
        self.analog_input_functions.append((index, function))

    def add_counters_function(self, index, function):
        self.counters_functions.append((index, function))

    def add_frozen_counters_function(self, index, function):
        self.frozen_counters_functions.append((index, function))
