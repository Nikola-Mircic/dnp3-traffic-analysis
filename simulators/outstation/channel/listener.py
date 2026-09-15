import logging
from pydnp3 import asiodnp3

_log = logging.getLogger(__name__)

class ChannelListener(asiodnp3.IChannelListener):
    """
        Override IChannelListener in this manner to implement application-specific channel behavior.
    """
    def __init__(self):
        super(ChannelListener, self).__init__()

    def OnStateChange(self, state):
        _log.debug('In ChannelListener.OnStateChange: state={}'.format(state))