//! Checks the Rust sn8 tools against outputs of the original Python tools.
//!
//! The Lenovo installer cannot be redistributed, so these tests need its path
//! in `KU1255_INSTALLER`; they are skipped (pass) when it is not set.
//! Expected hashes were produced with dissn8.py / assn8.py.

use std::collections::HashMap;
use std::path::Path;

use ku1255_core::{diff, format, installer, sn8, template};
use sha2::{Digest, Sha256};

const INSTALLER_SHA256: &str = "7116a3819ee094857d21e4671cb6cf953d582372126f0f6728f6b2421eda7bd4";
const FW_ORG_SHA256: &str = "286c7ea1a524456c0069ec9f3ebe7d4646665e07ed300f18b229f5cdbf6c9c55";
/// dissn8.py -c sn8f2288.cfg fw_org.bin
const FW_ORG_ASM_SHA256: &str = "a9e06df52b36800ff61a81ce45f4489cf6cb495be9dea4a7e632bc1c786e18bd";
/// assn8.py on the template rendered with default values
const FW_DEFAULT_BIN_SHA256: &str =
    "566541dad80550342169c108bef1b44bce34bea09a8d5fb3cd162cb77ad71b6a";

fn sha256(data: &[u8]) -> String {
    Sha256::digest(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn original_firmware() -> Option<Vec<u8>> {
    let Ok(path) = std::env::var("KU1255_INSTALLER") else {
        eprintln!("KU1255_INSTALLER not set; skipping");
        return None;
    };
    let installer = std::fs::read(path).expect("read installer");
    assert_eq!(sha256(&installer), INSTALLER_SHA256);
    let fw = installer::extract_fw_from_installer_to_vec(&installer).unwrap();
    assert_eq!(sha256(&fw), FW_ORG_SHA256);
    Some(fw)
}

#[test]
fn disassembly_matches_python() {
    let Some(fw) = original_firmware() else {
        return;
    };
    let asm = sn8::disassemble(&fw, &sn8::Chip::sn8f2288()).unwrap();
    assert_eq!(sha256(asm.as_bytes()), FW_ORG_ASM_SHA256);
}

#[test]
fn disassembly_reassembles_to_original() {
    let Some(fw) = original_firmware() else {
        return;
    };
    let asm = sn8::disassemble(&fw, &sn8::Chip::sn8f2288()).unwrap();
    let bin = sn8::assemble(&asm).unwrap();
    assert_eq!(sha256(&bin), FW_ORG_SHA256);
}

#[test]
fn default_template_assembles_like_python() {
    let Some(fw) = original_firmware() else {
        return;
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();

    let org_asm = sn8::disassemble(&fw, &sn8::Chip::sn8f2288()).unwrap();
    let fmt_asm = format::format_asm(&org_asm);
    let tmp_asm =
        diff::apply_diff(&fmt_asm, &read("template/diff.json"), &read("template/comments.txt")).unwrap();
    let mod_asm = template::render_template(&tmp_asm, &HashMap::new(), &HashMap::new()).unwrap();

    let bin = sn8::assemble(&mod_asm).unwrap();
    assert_eq!(sha256(&bin), FW_DEFAULT_BIN_SHA256);
}
