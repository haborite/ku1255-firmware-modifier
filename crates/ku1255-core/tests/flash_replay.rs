//! Replays I/O recorded from the original `flashsn8_gui.py` against a
//! simulated device (see `tests/data/record.py`), and checks that the Rust
//! flasher sends exactly the same packets and sleeps.

use std::collections::{HashSet, VecDeque};
use std::time::Duration;

use ku1255_core::flash::{self, Transport, TransportError};
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
struct Golden {
    image: ImageSpec,
    scenarios: Vec<Scenario>,
}

#[derive(Deserialize)]
struct ImageSpec {
    length: usize,
    lcg_seed: u64,
    canary_at: usize,
    code_options_at: usize,
}

#[derive(Deserialize)]
struct Device {
    timeouts: Vec<usize>,
}

#[derive(Deserialize)]
struct Scenario {
    name: String,
    device: Device,
    corrupt_canary: bool,
    result: String,
    responses: Vec<String>,
    trace_len: usize,
    trace_sha256: String,
}

struct Replay {
    responses: VecDeque<[u8; 8]>,
    timeouts: HashSet<usize>,
    sends: usize,
    trace: Vec<String>,
}

fn hex(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x}")).collect()
}

impl Transport for Replay {
    fn send(&mut self, data: &[u8; 8]) -> Result<(), TransportError> {
        let index = self.sends;
        self.sends += 1;
        if self.timeouts.contains(&index) {
            self.trace.push(format!("send-timeout {}", hex(data)));
            return Err(TransportError::Timeout);
        }
        self.trace.push(format!("send {}", hex(data)));
        Ok(())
    }

    fn recv(&mut self) -> Result<[u8; 8], TransportError> {
        let r = self
            .responses
            .pop_front()
            .ok_or_else(|| TransportError::Other("no recorded response left".into()))?;
        self.trace.push(format!("recv {}", hex(&r)));
        Ok(r)
    }

    fn sleep(&mut self, duration: Duration) {
        self.trace.push(format!("sleep {}", duration.as_millis()));
    }
}

fn synthetic_image(spec: &ImageSpec) -> Vec<u8> {
    let mut x = spec.lcg_seed;
    let mut img: Vec<u8> = (0..spec.length)
        .map(|_| {
            x = (x * 1103515245 + 12345) & 0x7fff_ffff;
            ((x >> 16) & 0xff) as u8
        })
        .collect();
    img[spec.canary_at..spec.canary_at + 2].copy_from_slice(&[0xaa, 0xaa]);
    img[spec.code_options_at..spec.code_options_at + 8]
        .copy_from_slice(&[0xf4, 0xff, 0x24, 0x79, 0x5a, 0xfa, 0x58, 0x45]);
    img
}

fn parse8(s: &str) -> [u8; 8] {
    let mut out = [0; 8];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap();
    }
    out
}

#[test]
fn replays_python_flasher_traces() {
    let golden: Golden = serde_json::from_str(include_str!("data/flash_golden.json")).unwrap();
    let original = synthetic_image(&golden.image);
    for scenario in &golden.scenarios {
        let mut image = original.clone();
        if scenario.corrupt_canary {
            image[golden.image.canary_at] = 0;
        }
        let mut replay = Replay {
            responses: scenario.responses.iter().map(|r| parse8(r)).collect(),
            timeouts: scenario.device.timeouts.iter().copied().collect(),
            sends: 0,
            trace: Vec::new(),
        };
        let result = flash::check_image(&image, &original)
            .and_then(|checked| flash::flash(&mut replay, &checked, &mut |_| {}));
        let trace: String = replay.trace.iter().map(|l| format!("{l}\n")).collect();
        let digest: String = Sha256::digest(trace.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        let name = &scenario.name;
        assert_eq!(
            result.is_ok(),
            scenario.result == "ok",
            "{name}: {result:?}"
        );
        assert_eq!(
            replay.trace.len(),
            scenario.trace_len,
            "{name}: trace length"
        );
        assert_eq!(digest, scenario.trace_sha256, "{name}: trace differs");
        assert!(replay.responses.is_empty(), "{name}: unused responses");
    }
}

#[test]
fn rejects_images_unrelated_to_original() {
    let spec = ImageSpec {
        length: 0x6000,
        lcg_seed: 1,
        canary_at: 0x4ffe,
        code_options_at: 0x5ff8,
    };
    let original = synthetic_image(&spec);

    let mut prefix_changed = original.clone();
    prefix_changed[0x11e] ^= 1;
    assert!(
        flash::check_image(&prefix_changed, &original)
            .unwrap_err()
            .contains("prefix")
    );

    let mut after_prefix = original.clone();
    after_prefix[0x11f] ^= 1;
    assert!(flash::check_image(&after_prefix, &original).is_ok());

    assert!(
        flash::check_image(&original[..0x5000], &original)
            .unwrap_err()
            .contains("size")
    );
    assert!(
        flash::check_image(&original[..0x5000], &original[..0x5000])
            .unwrap_err()
            .contains("Invalid image length")
    );
}
