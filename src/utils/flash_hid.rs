use hidapi::{HidApi, HidDevice, HidError};
use ku1255_core::flash::{DEVICE_IDS, Transport, TransportError};

/// Keyboard interface carrying the 8-byte feature report (no report ID).
const FLASH_INTERFACE: i32 = 0;

/// HID feature-report transport to the keyboard (USB interface 0, report ID 0).
pub struct HidTransport {
    device: HidDevice,
}

impl HidTransport {
    /// Open the single connected KU-1255 (normal or flasher mode).
    pub fn open() -> Result<HidTransport, String> {
        let api = HidApi::new().map_err(|e| format!("Failed to initialize HID: {e}"))?;
        // Do not seize the keyboard from the OS; feature reports work without it.
        #[cfg(target_os = "macos")]
        api.set_open_exclusive(false);

        let mut paths: Vec<_> = api
            .device_list()
            .filter(|d| DEVICE_IDS.contains(&(d.vendor_id(), d.product_id())))
            .filter(|d| d.interface_number() == FLASH_INTERFACE)
            .map(|d| d.path().to_owned())
            .collect();
        paths.dedup();
        if paths.len() != 1 {
            return Err(format!("{} device(s) found. Connect exactly one keyboard.", paths.len()));
        }
        let device = api.open_path(&paths[0]).map_err(|e| {
            let hint = if cfg!(target_os = "linux") {
                " (install the udev rule to allow access without root)"
            } else {
                ""
            };
            format!("Failed to open the keyboard: {e}{hint}")
        })?;
        Ok(HidTransport { device })
    }
}

fn transport_error(e: HidError) -> TransportError {
    let message = e.to_string();
    let lower = message.to_lowercase();
    // hidapi does not expose a dedicated timeout error; the platforms report
    // "timed out" (Linux ETIMEDOUT, macOS kIOReturnTimeout) or Windows error 121.
    if lower.contains("timed out") || lower.contains("timeout") || lower.contains("(0x00000079)") {
        TransportError::Timeout
    } else {
        TransportError::Other(message)
    }
}

impl Transport for HidTransport {
    fn send(&mut self, data: &[u8; 8]) -> Result<(), TransportError> {
        let mut report = [0u8; 9]; // report ID 0, then 8 data bytes
        report[1..].copy_from_slice(data);
        self.device.send_feature_report(&report).map_err(transport_error)
    }

    fn recv(&mut self) -> Result<[u8; 8], TransportError> {
        // hidapi puts the report ID in the first byte on every platform.
        let mut report = [0u8; 9];
        // The reported length is not used: in normal mode the keyboard answers
        // with no data (Windows then reports 1 byte), which the protocol treats
        // as an unexpected response, like the Python flasher did with libusb.
        // Unfilled bytes stay zero and can never match an expected prefix.
        self.device.get_feature_report(&mut report).map_err(transport_error)?;
        Ok(report[1..].try_into().expect("8 bytes"))
    }
}
