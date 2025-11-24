use std::process::Command;

const DISSN8_PATH: &str =  "sn8/dissn8/dissn8.dist/dissn8.exe";
const ASSN8_PATH:  &str =  "sn8/assn8/assn8.dist/assn8.exe";
const CFG_PATH:    &str =  "sn8/dissn8/dissn8.dist/sn8/sn8f2288.cfg";

pub fn run_dissn8(fw_bin: &str, out_asm: &str) -> std::io::Result<()> {
    let status = Command::new(DISSN8_PATH)
        .args(["-c", CFG_PATH, fw_bin, "-o", out_asm])
        .status()?;

    if !status.success() {
        eprintln!("dissn8 failed with {:?}", status);
    } else {
        println!("Generated {}", out_asm);
    }
    Ok(())
}

pub fn run_assn8(fw_asm: &str, out_bin: &str) -> std::io::Result<()> {
    let status = Command::new(ASSN8_PATH)
        .args([fw_asm, "-o", out_bin])
        .status()?;

    if !status.success() {
        eprintln!("assn8 failed with {:?}", status);
    } else {
        println!("Generated {}", out_bin);
    }
    Ok(())
}