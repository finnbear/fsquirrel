use std::ffi::OsString;
use tempfile::NamedTempFile;

#[test]
fn str_test() {
    let file = NamedTempFile::new().unwrap();
    let path = file.path();

    let key_1 = String::from("key");
    let value_1 = "hello".repeat(32);

    let key_1 = key_1.as_str();
    let value_1 = value_1.as_bytes();

    // Make sure these compile.
    let _ = fsquirrel::get(path, key_1).unwrap();
    fsquirrel::set(path, key_1, value_1).unwrap();
    fsquirrel::remove(path, key_1).unwrap();
}

#[test]
fn os_str_test() {
    let file = NamedTempFile::new().unwrap();
    let path = file.path();

    for i in 0..109 {
        let key = OsString::from(format!("key{i}"));
        let value = format!("hello{i}").repeat(32);

        let key = &key;
        let value = &value;

        let result = fsquirrel::get(path, key);
        assert!(result.as_ref().unwrap().is_none(), "{:?}", result);

        fsquirrel::set(path, key, value).unwrap();

        let result = fsquirrel::get(path, key);
        assert!(
            result.as_ref().unwrap().as_ref().unwrap() == value.as_bytes(),
            "{:?}",
            result
        );

        if i % 2 == 1 && i < 10 {
            continue;
        }

        fsquirrel::remove(path, key).unwrap();

        let result = fsquirrel::get(path, key);
        assert!(result.as_ref().unwrap().is_none(), "{:?}", result);
    }
}
