use std::io::{Error, Result};

use windows::core::PCWSTR;
use windows::Win32::Storage::FileSystem::{GetLogicalDrives, GetDriveTypeW};

// from https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getdrivetypew
const DRIVE_CDROM: u32 = 5;

pub fn get_drive_letter() -> Result<Vec<String>> {
    let mut res = vec![];
    let drives = unsafe { GetLogicalDrives() };

    if drives == 0 {
        return Err(Error::last_os_error());
    }

    for i in 0..26 {
        if drives & (1 << i) != 0 {
            let drive_char = (b'A' + i as u8) as char;
            let drive_letter = format!("{}:\\", drive_char);

            // Convert to UTF-16 and null-terminate for Windows API
            let wide_string: Vec<u16> = drive_letter.encode_utf16().chain(std::iter::once(0)).collect();
            let pcwstr = PCWSTR::from_raw(wide_string.as_ptr());

            let drive_type = unsafe { GetDriveTypeW(pcwstr) };
            if drive_type == DRIVE_CDROM {
                let drive_handle = format!(r"\\.\{drive_char}:");
                res.push(drive_handle);
            }
        }
    }

    Ok(res)
}
