use coolfindpattern::pattern;
use simplelog::Config;

const RESOLVE_FOLDER: &'static str = r#"C:\Program Files\Blackmagic Design\DaVinci Resolve"#;

const PATCHES: &'static [(&'static [Option<u8>], &'static [u8])] = &[
    (
        pattern!(
            0x48, 0x89, 0x5C, 0x24, _, 0x48, 0x89, 0x74, 0x24, _, 0x57, 0x48, 0x83, 0xEC, 0x60,
            0x48, 0x8B, 0x05, _, _, _, _, 0x48, 0x33, 0xC4, 0x48, 0x89, 0x44, 0x24, _, 0x48, 0x8B,
            0xF1, 0x89, 0x51
        ),
        &[0x48, 0x31, 0xC0, 0x48, 0xFF, 0xC0, 0xC3],
    ),
    (
        pattern!(
            0x40, 0x53, 0x48, 0x83, 0xEC, 0x20, 0x89, 0x51, 0x20, 0x48, 0x8B, 0xD9, 0xC6, 0x41,
            0x24, 0x00, 0x83, 0xEA, 0x01, 0x74
        ),
        &[0xB0, 0x01, 0xC3],
    ),
];

#[derive(Debug, thiserror::Error)]
enum PatchError {
    #[error("Resolve could not be located.")]
    ResolveNotFound,
    #[error("A pattern that should only be present once was instead present {0} times.")]
    SignatureOccurrenceMismatch(usize),
    #[error("This version of Resolve is either not compatible or was already patched.")]
    NoSignatureFound,
    #[error("Unable to backup Resolve.exe.")]
    BackupFailed,
    #[error("Unable to write patched Resolve.exe back.")]
    WriteFailed,
}

fn patch() -> Result<(), PatchError> {

    let Ok(mut data) = std::fs::read(&format!("{RESOLVE_FOLDER}\\Resolve.exe")) else {
        Err(PatchError::ResolveNotFound)?
    };

    let mut patches = false;

    for (sig, replacement) in PATCHES {
        let searcher = coolfindpattern::PatternSearcher::new(&data, sig);

        let occs: Vec<usize> = searcher.collect();

        match occs.len() {
            0 => {}
            1 => {
                let addr = occs[0];

                data[addr..addr + replacement.len()].copy_from_slice(replacement);

                patches = true;
            }
            _ => {
                Err(PatchError::SignatureOccurrenceMismatch(occs.len()))?
            }
        }
    }

    if !patches {
        Err(PatchError::NoSignatureFound)?
    }

    let Ok(_) = std::fs::copy(
        &format!("{RESOLVE_FOLDER}\\Resolve.exe"),
        &format!("{RESOLVE_FOLDER}\\Resolve.exe.bak"),
    ) else {
        Err(PatchError::BackupFailed)?
    };

    let Ok(_) = std::fs::write(&format!("{RESOLVE_FOLDER}\\Resolve.exe"), data) else {
        Err(PatchError::WriteFailed)?
    };

    Ok(())
}

fn main() {
    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, Config::default());

    log::info!("attempting to patch resolve!");

    match patch() {
        Ok(_) => {
            log::info!("successfully patched!");
        }
        Err(e) => {
            log::error!("failed to patch resolve: {e:#?}")
        }
    }

    std::thread::sleep(std::time::Duration::from_secs(5));
}
