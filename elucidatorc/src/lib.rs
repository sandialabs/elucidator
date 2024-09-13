use elucidator::{
    designation::DesignationSpecification,
    error::ElucidatorError,
};

use std::{
    collections::HashMap,
    ffi::CStr,
    os::raw::{c_char, c_int},
    sync::{LazyLock, RwLock},
};

type Dmap = LazyLock<RwLock<HashMap<String, DesignationSpecification>>>;
static DESIGNATION_MAP: Dmap = LazyLock::new(|| {
    RwLock::new(HashMap::new())
});

#[no_mangle]
pub extern "C" fn add_designation_from_text(name: *const c_char, spec: *const c_char) -> c_int {
    println!("Stuff");
    let name = String::from_utf8_lossy(
        unsafe { CStr::from_ptr(name) }.to_bytes()
    );
    let spec = String::from_utf8_lossy(
        unsafe { CStr::from_ptr(spec) }.to_bytes()
    );
    let designation = match DesignationSpecification::from_text(&spec) {
        Ok(o) => o,
        Err(e) => { 
            eprintln!("{e}");
            return 1;
        },
    };
    println!("Designation parsed as {designation:#?}");
    DESIGNATION_MAP
        .write()
        .unwrap()
        .insert(name.to_string(), designation);
    0
}
