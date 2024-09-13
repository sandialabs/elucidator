use elucidator::{
    designation::DesignationSpecification,
    error::ElucidatorError,
};

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    ffi::CStr,
    os::raw::{c_char, c_int},
    ptr,
    sync::{
        LazyLock, Mutex, RwLock
    },
};

type Dmap = LazyLock<RwLock<HashMap<DesignationHandle, DesignationSpecification>>>;
static DESIGNATION_MAP: Dmap = LazyLock::new(|| {
    RwLock::new(HashMap::new())
});

static HANDLE_NUM: Mutex<u32> = Mutex::new(1);

pub trait Handle: Hash + Eq { 
    fn get_new() -> Self;
    fn id(&self) -> u32;
}

macro_rules! impl_handle {
    ($($tt:ty), *) => {
        $(
            impl Hash for $tt {
                fn hash<H>(&self, state: &mut H)
                    where H: Hasher
                {
                    self.hdl.hash(state);
                }
            }
            impl Handle for $tt {
                fn get_new() -> Self {
                    let mut n = HANDLE_NUM.lock().unwrap();
                    let hdl = *n;
                    *n += 1;
                    Self { hdl: hdl.clone() }
                }
                fn id(&self) -> u32 { self.hdl }
            }
        )*
    };
}

#[repr(C)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignationHandle {
    hdl: u32
}

impl_handle!(DesignationHandle);

#[no_mangle]
pub extern "C" fn get_designation_from_text(name: *const c_char, spec: *const c_char) -> DesignationHandle {
    let name = String::from_utf8_lossy(
        unsafe { CStr::from_ptr(name) }.to_bytes()
    );
    let spec = String::from_utf8_lossy(
        unsafe { CStr::from_ptr(spec) }.to_bytes()
    );
    let designation = match DesignationSpecification::from_text(&spec) {
        Ok(o) => o,
        Err(e) => { 
            panic!("{e}");
        },
    };
    let dh = DesignationHandle::get_new();
    DESIGNATION_MAP.write().unwrap().insert(dh.clone(), designation);
    dh
}

#[no_mangle]
pub extern "C" fn print_designation(handle: *const DesignationHandle) {
    let map = DESIGNATION_MAP.read().unwrap();
    unsafe {
        let spec = map.get(&(*handle));
        println!("{spec:#?}");
    };
}
