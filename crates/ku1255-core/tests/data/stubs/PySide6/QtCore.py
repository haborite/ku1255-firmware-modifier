class _Bound:
    def __init__(self, log, name): self.log, self.name = log, name
    def emit(self, *a): self.log.append([self.name] + list(a))
    def connect(self, f): pass
class Signal:
    def __init__(self, *types): pass
    def __set_name__(self, owner, name): self.name = name
    def __get__(self, obj, owner=None):
        if obj is None: return self
        return _Bound(obj._signal_log, self.name)
class QThread:
    def __init__(self): self._signal_log = []
class Qt: WindowCloseButtonHint = 0
