#[derive(Debug, serde::Serialize)]
struct Output {
    result: i32,
    description: String,
}
#[no_mangle]
pub fn execute(ptr: *const u8, len: usize) -> *mut i8 {
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    let string = std::str::from_utf8(slice).unwrap();
    let output_data = if let Ok(value) = serde_json::from_str::<Vec<i32>>(string) {
        let data_output = Output {
            result: value.iter().sum(),
            description: String::from("This node performs an addition"),
        };
        serde_json::to_string(&data_output).unwrap()
    } else {
        String::from("Numbers to add not found")
    };
    let c_str = std::ffi::CString::new(output_data).unwrap();
    c_str.into_raw()
}
