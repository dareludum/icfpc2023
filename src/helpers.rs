use std::ffi::OsStr;

pub fn os_str_to_str(str: Option<&OsStr>) -> String {
    str.expect("OsStr is None")
        .to_str()
        .expect("Can't convert OsStr to String")
        .to_string()
}
