use std::{ffi::OsString, fs, io::ErrorKind, process::Command};
use tempfile::{NamedTempFile, TempDir};

#[test]
fn not_found_test() {
    let result = fsquirrel::get("does_not_exist_1", "foo");
    assert!(
        result
            .as_ref()
            .err()
            .filter(|e| e.kind() == ErrorKind::NotFound)
            .is_some(),
        "{result:?}"
    );

    let result = fsquirrel::set("does_not_exist_2", "foo", "bar");
    assert!(
        result
            .as_ref()
            .err()
            .filter(|e| e.kind() == ErrorKind::NotFound)
            .is_some(),
        "{result:?}"
    );

    let result = fsquirrel::remove("does_not_exist_3", "foo");
    assert!(
        result
            .as_ref()
            .err()
            .filter(|e| e.kind() == ErrorKind::NotFound)
            .is_some(),
        "{result:?}"
    );
}

#[test]
fn str_test() {
    let file = NamedTempFile::new().unwrap();
    let path = file.path();

    let key = String::from("key");
    let value = "hello".repeat(32);

    let key = key.as_str();
    let value = value.as_bytes();

    // Make sure these compile.
    let _ = fsquirrel::get(path, key).unwrap();
    fsquirrel::set(path, key, value).unwrap();
    fsquirrel::remove(path, key).unwrap();
    fsquirrel::list(path).unwrap();
}

#[test]
fn os_str_test() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("file1.txt");
    let path = &path;

    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);

    fs::write(path, "nothing to see here").unwrap();

    let mut expected_count = 0;
    for i in 0..100 {
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);

        let key = OsString::from(format!("key{i}"));
        let value = format!("hello{i}").repeat(32);

        let key = &key;
        let value = &value;

        let result = fsquirrel::get(path, key);
        assert!(result.as_ref().unwrap().is_none(), "{result:?}");

        let iter = fsquirrel::list(path).unwrap();
        //assert_eq!(
        iter.inspect(|i| {
            let s = i.as_ref().unwrap();
            let s = s.to_str().unwrap();
            println!("{s}");
            //s.strip_prefix("key").expect(s).parse::<usize>().expect(s);
        })
        .count(); //,
                  //    expected_count
                  //);

        fsquirrel::set(path, key, value).unwrap();
        expected_count += 1;

        let result = fsquirrel::get(path, key);
        assert!(
            result.as_ref().unwrap().as_ref().unwrap() == value.as_bytes(),
            "{result:?}",
        );

        if i % 2 == 1 && i < 10 {
            continue;
        }

        fsquirrel::remove(path, key).unwrap();
        expected_count -= 1;

        let result = fsquirrel::get(path, key);
        assert!(result.as_ref().unwrap().is_none(), "{result:?}");
    }
}

#[test]
fn fs_copy_test() {
    let dir = TempDir::new().unwrap();
    let path1 = dir.path().join("file1.txt");
    let path1 = &path1;
    let path2 = dir.path().join("file2.txt");
    let path2 = &path2;

    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);

    fs::write(path1, "nothing to see here").unwrap();

    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);

    let key = OsString::from(String::from("key"));
    let value = "hello".repeat(32);

    let key = &key;
    let value = &value;

    fsquirrel::set(path1, key, value).unwrap();

    if cfg!(any(windows, target_vendor = "apple")) {
        fs::copy(path1, path2).unwrap();
    } else {
        assert!(Command::new("/usr/bin/cp")
            .arg("--preserve=xattr")
            .arg(path1)
            .arg(path2)
            .spawn()
            .unwrap()
            .wait()
            .unwrap()
            .success());
    }

    let result = fsquirrel::get(path2, key);
    assert!(
        result.as_ref().unwrap().as_ref().unwrap() == value.as_bytes(),
        "{result:?}",
    );
}
