use std::{
    ffi::{OsStr, OsString},
    io::{self, Error, ErrorKind},
    path::Path,
};

#[cfg(windows)]
mod iter_windows;

/// Gets an extensible file attribute by `name`.
///
/// If `path` does not exist, a `NotFound` error is returned.
pub fn get<P: AsRef<Path>, N: AsRef<OsStr>>(path: P, name: N) -> io::Result<Option<Vec<u8>>> {
    #[allow(unused)]
    let (path, name) = (path.as_ref(), name.as_ref());

    #[cfg(unix)]
    return with_namespaced_name(name, |namespaced_name| {
        xattr::get_deref(path, namespaced_name)
    });

    #[cfg(windows)]
    return with_ads_path(path, name, |ads_path| {
        match std::fs::OpenOptions::new().read(true).open(ads_path) {
            // TOCTOU but the best we can do on Windows without fancy
            // winapi file opening that prevents deletion elsewhere.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && std::fs::exists(path)? => {
                Ok(None)
            }
            Err(e) => Err(e),
            Ok(mut file) => {
                use std::io::Read;
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                Ok(Some(buf))
            }
        }
    });

    #[allow(unreachable_code)]
    Err(Error::new(ErrorKind::Unsupported, "unsupported OS"))
}

/// Sets an extensible file attribute by name by creating or overwriting.
///
/// # Errors
///
/// If `path` does not exist, a `NotFound` error is returned.
///
/// Each platform may impose a limit on number of attributes, length
/// and validity of `name`, and length of `value`. An error will be
/// returned if such a limit is exceeded.
pub fn set<P: AsRef<Path>, N: AsRef<OsStr>, V: AsRef<[u8]>>(
    path: P,
    name: N,
    value: V,
) -> io::Result<()> {
    #[allow(unused)]
    let (path, name, value) = (path.as_ref(), name.as_ref(), value.as_ref());

    #[cfg(unix)]
    return with_namespaced_name(name, |namespaced_name| {
        xattr::set_deref(path, namespaced_name, value)
    });

    #[cfg(windows)]
    return {
        if !std::fs::exists(path)? {
            // TOCTOU but the best we can do on Windows without fancy
            // winapi file opening that prevents deletion elsewhere.
            return Err(Error::new(ErrorKind::NotFound, "file does not exist"));
        }
        with_ads_path(path, name, |ads_path| {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(ads_path)?;
            file.write_all(value)?;
            file.sync_all()?;
            Ok(())
        })
    };

    #[allow(unreachable_code)]
    Err(Error::new(ErrorKind::Unsupported, "unsupported OS"))
}

/// Removes an extensible file attribute by name.
///
/// # Errors
///
/// If `path` does not exist, a `NotFound` error is returned.
pub fn remove<P: AsRef<Path>, N: AsRef<OsStr>>(path: P, name: N) -> io::Result<()> {
    #[allow(unused)]
    let (path, name) = (path.as_ref(), name.as_ref());

    #[cfg(unix)]
    return with_namespaced_name(name, |namespaced_name| {
        xattr::remove_deref(path, namespaced_name)
    });

    #[cfg(windows)]
    return {
        if !std::fs::exists(path)? {
            // TOCTOU but the best we can do on Windows without fancy
            // winapi file opening that prevents deletion elsewhere.
            return Err(Error::new(ErrorKind::NotFound, "file does not exist"));
        }
        with_ads_path(path, name, |ads_path| std::fs::remove_file(ads_path))
    };

    #[allow(unreachable_code)]
    Err(Error::new(ErrorKind::Unsupported, "unsupported OS"))
}

pub struct Attributes {
    #[cfg(windows)]
    inner: iter_windows::AttributesImpl,
    #[cfg(unix)]
    inner: std::iter::FilterMap<xattr::XAttrs, fn(OsString) -> Option<OsString>>,
}

impl Iterator for Attributes {
    type Item = io::Result<OsString>;

    fn next(&mut self) -> Option<Self::Item> {
        #[cfg(windows)]
        return self.inner.next();
        #[cfg(unix)]
        self.inner.next().map(Ok)
    }
}

pub fn list<P: AsRef<Path>>(path: P) -> io::Result<Attributes> {
    Ok(Attributes {
        #[cfg(windows)]
        inner: iter_windows::AttributesImpl::new(path.as_ref())?,
        #[cfg(unix)]
        inner: xattr::list(path)?.filter_map(|s| {
            const USER_NAMESPACE: &'static [u8] = b"user.";
            let bytes = s.as_encoded_bytes();
            if bytes.starts_with(USER_NAMESPACE) {
                Some(
                    // SAFETY: we split off a valid UTF-8 substring,
                    // so the remainder starts at the boundary of
                    // valid UTF-8.
                    unsafe {
                        OsStr::from_encoded_bytes_unchecked(
                            &bytes[USER_NAMESPACE.len()..bytes.len()],
                        )
                    }
                    .to_owned(),
                )
            } else {
                None
            }
        }),
    })
}

// Re-use a buffer for efficiency.
#[cfg(any(unix, windows))]
fn with_buffer<R>(inner: impl Fn(&mut std::ffi::OsString) -> R) -> R {
    thread_local! {
        // `OsString::new` isn't currently const, so use `Option`.
        static BUFFER: std::cell::RefCell<Option<std::ffi::OsString>>
            = const { std::cell::RefCell::new(None) };
    }

    BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        if buffer.is_none() {
            *buffer = Some(std::ffi::OsString::new());
        }

        let buffer = buffer.as_mut().unwrap();
        let ret = inner(buffer);
        buffer.clear();
        ret
    })
}

/// Passes the namespaced `name` into `inner`.
#[cfg(unix)]
fn with_namespaced_name<R>(name: &OsStr, inner: impl Fn(&OsStr) -> R) -> R {
    with_buffer(|buffer| {
        buffer.push("user.");
        buffer.push(name);
        inner(buffer)
    })
}

/// Passes the combined Alternate Data Stream path to `inner`.
#[cfg(windows)]
fn with_ads_path<R>(path: &Path, name: &OsStr, inner: impl Fn(&OsStr) -> R) -> R {
    with_buffer(|buffer| {
        buffer.push(path);
        buffer.push(":");
        buffer.push(name);
        inner(buffer)
    })
}
