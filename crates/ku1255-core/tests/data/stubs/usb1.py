REQUEST_TYPE_CLASS = 0x20
RECIPIENT_INTERFACE = 0x01
class USBError(Exception): pass
class USBErrorTimeout(USBError): pass
class USBErrorAccess(USBError): pass
class USBErrorIO(USBError): pass
class USBErrorNotFound(USBError): pass
class USBErrorNotSupported(USBError): pass
DEVICE = None
class USBContext:
    def __enter__(self): return self
    def __exit__(self, *a): pass
    def getDeviceIterator(self, skip_on_error=False): return [DEVICE]
