//! SN8F2288 USB flashing protocol.
//!
//! The USB access is abstracted by [`Transport`] so the protocol can be tested
//! without hardware.

use std::time::Duration;

/// USB IDs of the keyboard: flasher mode first, then normal mode.
pub const DEVICE_IDS: [(u16, u16); 2] = [(0x0c45, 0x7500), (0x17ef, 0x6047)];

const ERASE_BLOCK_LENGTH_WORDS: usize = 0x80;
const IMAGE_LENGTH: usize = 0x3000 * 2;
/// SN8 image format: 0x100-byte header before the plain image.
const SN8_HEADER_LENGTH: usize = 0x100;
const CANARY_ADDRESS_WORDS: usize = 0x27ff;
const CANARY_ADDRESS_BYTES: usize = CANARY_ADDRESS_WORDS * 2;
const CANARY: [u8; 2] = [0xaa, 0xaa];
/// No writes (neither erase nor program) at or above this address.
const FLASHER_BASE_ADDRESS_WORDS: usize = 0x2800;
const FLASHER_BASE_ADDRESS_BYTES: usize = FLASHER_BASE_ADDRESS_WORDS * 2;
const UNPROGRAMABLE_PREFIX_WORDS: usize = 8;
const UNPROGRAMABLE_PREFIX_BYTES: usize = UNPROGRAMABLE_PREFIX_WORDS * 2;
/// JMP 0x2800 reset vector, jumping to canary checker. All other words are cleared.
const FIRST_8_WORDS_CHECKSUM: u32 = 0x80 + 0xa8;
const ALL_ERASED_EXPECTED_CHECKSUM: u16 =
    (((FLASHER_BASE_ADDRESS_BYTES - UNPROGRAMABLE_PREFIX_BYTES) as u32 * 0xff
        + FIRST_8_WORDS_CHECKSUM)
        & 0xffff) as u16;
const ERASABLE_PAGE_COUNT: usize = FLASHER_BASE_ADDRESS_WORDS / ERASE_BLOCK_LENGTH_WORDS;
const CODE_OPTIONS_BYTES: usize = 0x2ffc * 2;
/// The modified image must match the original firmware up to this byte (inclusive).
const ORIGINAL_PREFIX_END: usize = 0x11e;

const _: () = assert!(ALL_ERASED_EXPECTED_CHECKSUM == 0xa138);
const _: () = assert!(ERASABLE_PAGE_COUNT == 0x50);

/// Upper bound for retrying timed-out transfers (Python retries forever).
const MAX_TIMEOUT_RETRIES: usize = 1000;

#[derive(Debug)]
pub enum TransportError {
    Timeout,
    Other(String),
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::Timeout => write!(f, "USB transfer timed out"),
            TransportError::Other(msg) => write!(f, "{msg}"),
        }
    }
}

/// 8-byte HID feature report exchange with the keyboard (interface 0, report ID 0).
pub trait Transport {
    /// SET_REPORT (feature)
    fn send(&mut self, data: &[u8; 8]) -> Result<(), TransportError>;
    /// GET_REPORT (feature)
    fn recv(&mut self) -> Result<[u8; 8], TransportError>;
    fn sleep(&mut self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Message(String),
    /// Bytes sent so far / offset of the last packet.
    Progress {
        sent: usize,
        last: usize,
    },
}

/// A modified firmware image that passed every offline safety check.
#[derive(Clone, Debug)]
pub struct CheckedImage {
    /// Words 0x0008..0x2800, as programmed.
    program: Vec<u8>,
    code_options: [u8; 8],
    expected_checksum: u16,
}

/// Offline checks, before touching the device:
/// - same size as the original firmware, and identical bytes 0x0000-0x011E
///   (this project's guard against flashing an unrelated image),
/// - valid image length, canary present (flashsn8).
pub fn check_image(modified: &[u8], original: &[u8]) -> Result<CheckedImage, String> {
    if modified.len() != original.len() {
        return Err(format!(
            "Safety check failed: size mismatch (original={}, target={}).",
            original.len(),
            modified.len()
        ));
    }
    if modified.len() <= ORIGINAL_PREFIX_END {
        return Err("Safety check failed: image too short for prefix comparison.".into());
    }
    if modified[..=ORIGINAL_PREFIX_END] != original[..=ORIGINAL_PREFIX_END] {
        return Err("Safety check failed: prefix region (0x0000-0x011E) mismatch.".into());
    }
    let image = match modified.len() {
        IMAGE_LENGTH => modified,
        n if n == IMAGE_LENGTH + SN8_HEADER_LENGTH => &modified[SN8_HEADER_LENGTH..],
        n => {
            return Err(format!(
                "Invalid image length: {n} bytes. Expected one of: {IMAGE_LENGTH:#x}, {:#x}",
                IMAGE_LENGTH + SN8_HEADER_LENGTH
            ));
        }
    };
    let code_options: [u8; 8] = image[CODE_OPTIONS_BYTES..].try_into().expect("8 bytes");
    let program = &image[UNPROGRAMABLE_PREFIX_BYTES..FLASHER_BASE_ADDRESS_BYTES];
    if program[CANARY_ADDRESS_BYTES - UNPROGRAMABLE_PREFIX_BYTES..] != CANARY {
        return Err(format!(
            "Canary missing. Add \".ORG {CANARY_ADDRESS_WORDS:#04x} DW 0xaaaa\" to source and rebuild."
        ));
    }
    assert_eq!(program.len() / 8, 0x9fe);
    let sum = program.iter().map(|b| *b as u32).sum::<u32>();
    Ok(CheckedImage {
        program: program.to_vec(),
        code_options,
        expected_checksum: ((FIRST_8_WORDS_CHECKSUM + sum) & 0xffff) as u16,
    })
}

#[derive(Debug)]
enum Error {
    UnexpectedResponse([u8; 8]),
    Transport(TransportError),
    Failed(String),
}

impl From<TransportError> for Error {
    fn from(e: TransportError) -> Self {
        Error::Transport(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnexpectedResponse(r) => write!(f, "Unexpected response: {}", hex(r)),
            Error::Transport(e) => write!(f, "{e}"),
            Error::Failed(msg) => write!(f, "{msg}"),
        }
    }
}

fn hex(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn packet(head: [u8; 4], tail: [u8; 4]) -> [u8; 8] {
    let mut p = [0; 8];
    p[..4].copy_from_slice(&head);
    p[4..].copy_from_slice(&tail);
    p
}

fn command(cmd: u8, sub: u8) -> [u8; 8] {
    packet([cmd, 0xaa, 0x55, sub], [0; 4])
}

fn words(a: u16, b: u16) -> [u8; 4] {
    let mut out = [0; 4];
    out[..2].copy_from_slice(&a.to_le_bytes());
    out[2..].copy_from_slice(&b.to_le_bytes());
    out
}

struct Flasher<'a, T: Transport> {
    t: &'a mut T,
    on_event: &'a mut dyn FnMut(Event),
}

impl<T: Transport> Flasher<'_, T> {
    fn message(&mut self, text: impl Into<String>) {
        (self.on_event)(Event::Message(text.into()));
    }

    fn recv(&mut self, expected: &[u8]) -> Result<[u8; 8], Error> {
        let result = self.t.recv()?;
        if !result.starts_with(expected) {
            return Err(Error::UnexpectedResponse(result));
        }
        Ok(result)
    }

    fn switch_to_flasher(&mut self) -> Result<(), Error> {
        self.message("Switching to flasher...");
        Ok(self
            .t
            .send(&[0xaa, 0x55, 0xa5, 0x5a, 0xff, 0x00, 0x33, 0xcc])?)
    }

    fn unlock_flash(&mut self) -> Result<(), Error> {
        self.message("Unlocking flash...");
        self.t.send(&command(0x01, 0))?;
        self.recv(&[0x01, 0xaa, 0x55, 0x00, 0x00, 0x03, 0x00, 0x00])?;
        self.t
            .send(&packet([0x02, 0xaa, 0x55, 0x00], [0x12, 0x34, 0x56, 0x78]))?;
        self.recv(&[0x02, 0xaa, 0x55, 0x00, 0xfa, 0xfa, 0xfa, 0xfa])?;
        Ok(())
    }

    fn flash_unlock_state(&mut self) -> Result<bool, Error> {
        self.message("Getting flash lock state...");
        self.t.send(&command(0x03, 0))?;
        Ok(self.recv(&[0x03, 0xaa, 0x55, 0x00])?[4..] == [0xfa; 4])
    }

    fn erase(&mut self, base_address_words: usize, page_count: usize) -> Result<(), Error> {
        if page_count < 1 || base_address_words & 127 != 0 {
            return Err(Error::Failed(format!(
                "Bad erase range {base_address_words}, {page_count}"
            )));
        }
        let last = base_address_words + page_count * ERASE_BLOCK_LENGTH_WORDS - 1;
        // Flasher does not protect itself, do it instead.
        if last >= FLASHER_BASE_ADDRESS_WORDS {
            return Err(Error::Failed("Refusing to erase flasher program".into()));
        }
        self.message(format!(
            "Erasing {base_address_words:#04x} to {last:#04x}..."
        ));
        let tail = words(base_address_words as u16, page_count as u16);
        Ok(self.t.send(&packet([0x04, 0xaa, 0x55, 0x00], tail))?)
    }

    fn checksum(&mut self) -> Result<u16, Error> {
        self.message("Getting 0x0000 to 0x27ff checksum...");
        self.t.send(&command(0x06, 0))?;
        let r = self.recv(&[0x06, 0xaa, 0x55, 0x00, 0xfa, 0xfa])?;
        Ok(u16::from_le_bytes([r[6], r[7]]))
    }

    fn code_options(&mut self) -> Result<[u8; 8], Error> {
        self.message("Retrieving code options...");
        let mut out = [0; 8];
        self.t.send(&command(0x09, 0))?;
        out[..4].copy_from_slice(&self.recv(&[0x09, 0xaa, 0x55, 0x00])?[4..]);
        self.t.send(&command(0x09, 1))?;
        out[4..].copy_from_slice(&self.recv(&[0x09, 0xaa, 0x55, 0x01])?[4..]);
        Ok(out)
    }

    fn reboot(&mut self) -> Result<(), Error> {
        self.message("Asking device to reboot...");
        // There is (should be) no answer
        Ok(self.t.send(&command(0x07, 0))?)
    }

    fn program(&mut self, base_address_words: usize, data: &[u8]) -> Result<(), Error> {
        // The flasher always writes 4 words / 8 bytes per packet.
        if data.len() % 8 != 0 {
            return Err(Error::Failed("Data length must be a multiple of 8.".into()));
        }
        let last_address = base_address_words + data.len() / 2 - 1;
        if last_address >= FLASHER_BASE_ADDRESS_WORDS {
            return Err(Error::Failed(
                "Refusing to overwrite flasher program".into(),
            ));
        }
        self.message("⚠⚠⚠ DO NOT unplug the keyboard during flashing! ⚠⚠⚠");
        self.message(format!(
            "Programming from {base_address_words:#04x} to {last_address:#04x}..."
        ));
        let tail = words(base_address_words as u16, (data.len() / 8) as u16);
        self.t.send(&packet([0x05, 0xaa, 0x55, 0x00], tail))?;
        self.recv(&[0x05, 0xaa, 0x55, 0x00, 0xfa, 0xfa, 0xfa, 0xfa])?;
        let last = data.len() - 8;
        for (i, chunk) in data.chunks_exact(8).enumerate() {
            (self.on_event)(Event::Progress { sent: i * 8, last });
            let chunk: &[u8; 8] = chunk.try_into().expect("8 bytes");
            let mut retries = 0;
            loop {
                match self.t.send(chunk) {
                    Ok(()) => {
                        // Flash packet is acked immediately, but firmware seems to clear
                        // the USB interrupt at a different time, causing the next
                        // transmission to be lost. Sleep for 1ms.
                        self.t.sleep(Duration::from_millis(1));
                        break;
                    }
                    Err(TransportError::Timeout) if retries < MAX_TIMEOUT_RETRIES => {
                        retries += 1;
                        self.message("Timed out, retrying");
                    }
                    Err(e) => return Err(e.into()),
                }
            }
        }
        Ok(())
    }

    fn run(&mut self, image: &CheckedImage) -> Result<(), Error> {
        let unlocked = match self.flash_unlock_state() {
            Ok(unlocked) => unlocked,
            Err(Error::UnexpectedResponse(_)) => {
                self.message("Not in flasher mode. Switching...");
                self.switch_to_flasher()?;
                self.flash_unlock_state()?
            }
            Err(e) => return Err(e),
        };
        if !unlocked {
            self.unlock_flash()?;
            if !self.flash_unlock_state()? {
                return Err(Error::Failed("Failed to unlock flash.".into()));
            }
        }

        if self.code_options()? != image.code_options {
            return Err(Error::Failed(
                "Code option mismatch between flash and image.".into(),
            ));
        }

        self.erase(0, ERASABLE_PAGE_COUNT)?;
        self.t.sleep(Duration::from_millis(2500));
        let mut retries = 0;
        let erased = loop {
            match self.checksum() {
                Err(Error::Transport(TransportError::Timeout)) if retries < MAX_TIMEOUT_RETRIES => {
                    retries += 1;
                }
                other => break other?,
            }
        };
        if erased != ALL_ERASED_EXPECTED_CHECKSUM {
            return Err(Error::Failed(format!(
                "Post-erase checksum mismatch: expected {ALL_ERASED_EXPECTED_CHECKSUM:#04x}, got {erased:#04x}"
            )));
        }

        self.program(UNPROGRAMABLE_PREFIX_WORDS, &image.program)?;
        let checksum = self.checksum()?;
        if checksum != image.expected_checksum {
            // Leave the flash erased, so the device boots into the flasher.
            self.message("Checksum mismatch after programming. Attempting re-erase.");
            self.erase(0, ERASABLE_PAGE_COUNT)?;
            return Err(Error::Failed(format!(
                "Post-program checksum mismatch: expected {:#04x}, got {checksum:#04x}",
                image.expected_checksum
            )));
        }

        self.message("Success!");
        self.reboot()?;
        self.t.sleep(Duration::from_millis(500));
        Ok(())
    }
}

/// Flash a checked image. `on_event` receives progress messages.
pub fn flash<T: Transport>(
    transport: &mut T,
    image: &CheckedImage,
    on_event: &mut dyn FnMut(Event),
) -> Result<(), String> {
    Flasher {
        t: transport,
        on_event,
    }
    .run(image)
    .map_err(|e| e.to_string())
}
