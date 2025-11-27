use std::process::Command;

const PYTHON_PATH_UNX: &str = "python/python-linux-embed-amd64/python.exe";
const PYTHON_PATH_WIN: &str = "python/python-win-embed-amd64/python.exe";
const DISSN8_PATH: &str = "sn8tools/dissn8.py";
const ASSN8_PATH: &str = "sn8tools/assn8.py";
const CFG_PATH: &str = "sn8tools/sn8/sn8f2288.cfg";
const FLASHER_PATH_UNX: &str = "sn8tools/flashsn8-gui.bin";
const FLASHER_PATH_WIN: &str = "sn8tools/flashsn8-gui.exe";

fn get_python_path() -> std::io::Result<&'static str> {
    if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        Ok(PYTHON_PATH_UNX)
    } else if cfg!(target_os = "windows") {                    
        Ok(PYTHON_PATH_WIN)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unsupported OS",
        ))
    }
}

pub fn run_dissn8(fw_bin: &str, out_asm: &str) -> std::io::Result<()> {
    let python_path = get_python_path()?;
    let status = Command::new(python_path)
        .args([DISSN8_PATH, "-c", CFG_PATH, fw_bin, "-o", out_asm])
        .status()?;

    if !status.success() {
        eprintln!("dissn8 failed with {:?}", status);
    } else {
        println!("Generated {}", out_asm);
    }
    Ok(())
}

pub fn run_assn8(fw_asm: &str, out_bin: &str) -> std::io::Result<()> {
    let python_path = get_python_path()?;
    let status = Command::new(python_path)
        .args([ASSN8_PATH, fw_asm, "-o", out_bin])
        .status()?;

    if !status.success() {
        eprintln!("assn8 failed with {:?}", status);
    } else {
        println!("Generated {}", out_bin);
    }
    Ok(())
}

fn get_flasher_path() -> std::io::Result<&'static str> {
    if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        Ok(FLASHER_PATH_UNX)
    } else if cfg!(target_os = "windows") {                    
        Ok(FLASHER_PATH_WIN)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unsupported OS",
        ))
    }
}

pub fn run_flashsn8_gui(fw_bin_path: &str) -> std::io::Result<()> {
    let flasher_path = get_flasher_path()?;
    let status = Command::new(flasher_path)
        .arg(fw_bin_path)
        .status()?;
    if !status.success() {
        eprintln!("flashsn8-gui failed with {:?}", status);
    } else {
        println!("Launched flashsn8-gui with {}", fw_bin_path);
    }
    Ok(())
}
