use elucidator::{
    designation::DesignationSpecification,
    error::ElucidatorError,
};

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    ffi::{CStr, CString},
    os::raw::{c_char, c_int},
    ptr,
    sync::{
        atomic::{AtomicU32, Ordering},
        LazyLock, RwLock
    },
};

type Dmap = LazyLock<RwLock<HashMap<DesignationHandle, DesignationSpecification>>>;
static DESIGNATION_MAP: Dmap = LazyLock::new(|| {
    RwLock::new(HashMap::new())
});

type Emap = LazyLock<RwLock<HashMap<ErrorHandle, ElucidatorError>>>;
static ERROR_MAP: Emap = LazyLock::new(|| {
    RwLock::new(HashMap::new())
});

static HANDLE_NUM: AtomicU32 = AtomicU32::new(1);

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
                    let hdl = HANDLE_NUM.fetch_add(1, Ordering::SeqCst);
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

#[repr(C)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErrorHandle {
    hdl: u32
}

impl_handle!(ErrorHandle);

/// Create a designation from a given name and specification. On success, the function will return
/// 0 and place a valid handle into the pointer provided for dh. On failure, an error handle will
///   be placed into the pointer provided for eh and exit will be nonzero.
#[no_mangle]
pub extern "C" fn get_designation_from_text(
    spec: *const c_char,
    dh: *mut DesignationHandle,
    eh: *mut ErrorHandle,
) -> c_int {
    let spec = String::from_utf8_lossy(
        unsafe { CStr::from_ptr(spec) }.to_bytes()
    );
    let designation = match DesignationSpecification::from_text(&spec) {
        Ok(o) => o,
        Err(e) => {
            unsafe {
                *eh = ErrorHandle::get_new();
                ERROR_MAP.write().unwrap().insert((*eh).clone(), e);
            }
            return 1;
        },
    };
    unsafe {
        *dh = DesignationHandle::get_new();
        DESIGNATION_MAP.write().unwrap().insert((*dh).clone(), designation);
    }
    0
}

/// Get a string based on the provided handle. If the handle cannot be foundor is NULL, the
/// returned string will be NULL. You must free the returned pointer.
#[no_mangle]
pub extern "C" fn get_error_string(eh: *const ErrorHandle) -> *mut c_char {
    unsafe {
        match ERROR_MAP.read().unwrap().get(&*eh) {
            Some(e) => {
                CString::new(format!("{e}").as_str()).unwrap().into_raw()
            },
            None => ptr::null_mut::<c_char>(),
        }
    }
}

#[no_mangle]
pub extern "C" fn print_designation(handle: *const DesignationHandle) {
    let map = DESIGNATION_MAP.read().unwrap();
    unsafe {
        let spec = map.get(&(*handle));
        println!("{spec:#?}");
    };
}
