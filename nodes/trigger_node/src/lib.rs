#[derive(Debug, serde::Serialize)]
struct Output {
    description: String,
}

#[no_mangle]
pub fn execute(_ptr: *const u8, _len: usize) -> *mut i8 {
    // let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    // let string = std::str::from_utf8(slice).unwrap();

    let data_out = Output {
        description: String::from("Trigger is executed"),
    };

    let output_data = serde_json::to_string(&data_out).unwrap();

    let c_str = std::ffi::CString::new(output_data).unwrap();
    c_str.into_raw()
}
