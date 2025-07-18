use std::{
    ffi::OsString,
    io,
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::Path,
};
use windows::{core::*, Win32::Foundation::*, Win32::Storage::FileSystem::*};

pub(crate) struct AttributesImpl {
    handle: HANDLE,
    first: bool,
    find_data: WIN32_FIND_STREAM_DATA,
}

impl AttributesImpl {
    pub fn new(path: &Path) -> io::Result<Self> {
        let wide_path: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let mut find_data = WIN32_FIND_STREAM_DATA::default();

        let handle = unsafe {
            FindFirstStreamW(
                PCWSTR(wide_path.as_ptr()),
                FindStreamInfoStandard,
                &mut find_data as *mut WIN32_FIND_STREAM_DATA as *mut _,
                0,
            )
        }
        .map_err(|e| io::Error::from_raw_os_error(e.code().0))?;

        Ok(Self {
            handle,
            first: true,
            find_data,
        })
    }
}

impl Iterator for AttributesImpl {
    type Item = io::Result<OsString>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.handle == INVALID_HANDLE_VALUE {
            return None;
        }

        if !self.first {
            if let Err(err) = unsafe {
                FindNextStreamW(
                    self.handle,
                    &mut self.find_data as *mut WIN32_FIND_STREAM_DATA as *mut _,
                )
            } {
                unsafe {
                    FindClose(self.handle);
                }
                self.handle = INVALID_HANDLE_VALUE;
                return if err.code() == ERROR_HANDLE_EOF.to_hresult() {
                    None
                } else {
                    Some(Err(io::Error::from_raw_os_error(err.code().0)))
                };
            }
        } else {
            self.first = false;
        }

        let stream_name = OsString::from_wide(
            &self.find_data.cStreamName[0..self
                .find_data
                .cStreamName
                .iter()
                .position(|c| *c == 0)
                .unwrap_or(self.find_data.cStreamName.len())],
        );

        Some(Ok(stream_name))
    }
}

impl Drop for AttributesImpl {
    fn drop(&mut self) {
        if self.handle != INVALID_HANDLE_VALUE {
            unsafe {
                FindClose(self.handle);
            }
        }
    }
}
