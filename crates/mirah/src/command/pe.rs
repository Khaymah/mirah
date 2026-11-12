const PE_POINTER_AT: usize = 0x3c;
const PE_SIGNATURE: &[u8] = b"PE\0\0";
const SUBSYSTEM_FROM_SIGNATURE: usize = 4 + 20 + 68;
const WINDOWS_GUI: u16 = 2;
const HEADER_BYTES: usize = 4096;

fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes([*bytes.get(at)?, *bytes.get(at + 1)?]))
}

pub(crate) fn is_windowed(header: &[u8]) -> bool {
    let Some(pe) = header
        .get(PE_POINTER_AT..PE_POINTER_AT + 4)
        .and_then(|b| b.try_into().ok())
        .map(u32::from_le_bytes)
        .and_then(|p| usize::try_from(p).ok())
    else {
        return false;
    };
    header.starts_with(b"MZ")
        && header.get(pe..pe + PE_SIGNATURE.len()) == Some(PE_SIGNATURE)
        && u16_at(header, pe + SUBSYSTEM_FROM_SIGNATURE) == Some(WINDOWS_GUI)
}

#[cfg(windows)]
pub(crate) fn file_is_windowed(path: &std::path::Path) -> bool {
    use std::io::Read;
    let mut header = Vec::with_capacity(HEADER_BYTES);
    std::fs::File::open(path)
        .and_then(|f| f.take(HEADER_BYTES as u64).read_to_end(&mut header))
        .is_ok_and(|_| is_windowed(&header))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(subsystem: u16) -> Vec<u8> {
        let pe = 0x80;
        let mut bytes = vec![0u8; HEADER_BYTES];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[PE_POINTER_AT..PE_POINTER_AT + 4].copy_from_slice(&(pe as u32).to_le_bytes());
        bytes[pe..pe + 4].copy_from_slice(PE_SIGNATURE);
        let at = pe + SUBSYSTEM_FROM_SIGNATURE;
        bytes[at..at + 2].copy_from_slice(&subsystem.to_le_bytes());
        bytes
    }

    #[test]
    fn a_windowed_exe_is_told_apart_from_a_console_one() {
        assert!(is_windowed(&image(2)));
        assert!(!is_windowed(&image(3)));
        assert!(!is_windowed(b"#!/bin/sh\necho hi\n"));
        assert!(!is_windowed(&[]));
    }
}
