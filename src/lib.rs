use std::{ffi::OsStr, io, path::Path};

/// Gets an extensible file attribute by name.
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
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
            Ok(mut file) => {
                use std::io::Read;
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                Ok(Some(buf))
            }
        }
    });

    #[cfg(not(any(unix, windows)))]
    return Err(Error::new(ErrorKind::Unsupported, "unsupported OS"));
}

/// Sets an extensible file attribute by name by creating or overwriting.
///
/// # Limits
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
    return with_ads_path(path, name, |ads_path| {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(ads_path)?;
        file.write_all(value)?;
        file.sync_all()?;
        Ok(())
    });

    #[cfg(not(any(unix, windows)))]
    return Err(Error::new(ErrorKind::Unsupported, "unsupported OS"));
}

/// Removes an extensible file attribute by name.
pub fn remove<P: AsRef<Path>, N: AsRef<OsStr>>(path: P, name: N) -> io::Result<()> {
    #[allow(unused)]
    let (path, name) = (path.as_ref(), name.as_ref());

    #[cfg(unix)]
    return with_namespaced_name(name, |namespaced_name| {
        xattr::remove_deref(path, namespaced_name)
    });

    #[cfg(windows)]
    return with_ads_path(path, name, |ads_path| std::fs::remove_file(ads_path));

    #[cfg(not(any(unix, windows)))]
    return Err(Error::new(ErrorKind::Unsupported, "unsupported OS"));
}

// Re-use a buffer for efficiency.
#[cfg(any(unix, windows))]
fn with_buffer<R>(inner: impl Fn(&mut std::ffi::OsString) -> R) -> R {
    thread_local! {
        // `OsString::new` isn't currently const, so use `Option`.
        static BUFFER: std::cell::RefCell<Option<std::ffi::OsString>>
            = std::cell::RefCell::new(None);
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
