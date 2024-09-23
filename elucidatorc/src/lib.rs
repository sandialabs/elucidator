use elucidator::{
    designation::DesignationSpecification,
    error::ElucidatorError,
};

use elucidator_db::{
    error,
    backends::{sqlite::SqlDatabase, rtree::RTreeDatabase},
    database::Database,
};

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    ffi::{CStr, CString},
    os::raw::c_char,
    ptr,
    sync::{
        atomic::{AtomicU32, Ordering},
        LazyLock, RwLock
    },
};

type Emap = LazyLock<RwLock<HashMap<ErrorHandle, ElucidatorError>>>;
static ERROR_MAP: Emap = LazyLock::new(|| {
    RwLock::new(HashMap::new())
});

type Smap = LazyLock<RwLock<HashMap<SessionHandle, RTreeDatabase>>>;
static SESSION_MAP: Smap = LazyLock::new(|| {
    RwLock::new(HashMap::new())
});

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum DatabaseKind {
    ELUCIDATOR_RTREE,
}

static HANDLE_NUM: AtomicU32 = AtomicU32::new(1);

pub trait Handle: Hash { 
    fn get_new() -> Self;
    fn id(&self) -> u32;
}

macro_rules! impl_handle {
    ($($tt:ty), *) => {
        $(
            impl PartialEq for $tt {
                fn eq(&self, other: &Self) -> bool {
                    self.hdl == other.hdl
                }
            }
            impl Eq for $tt {}
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
#[derive(Clone, Debug, Hash)]
pub struct ErrorHandle {
    hdl: u32
}

impl_handle!(ErrorHandle);

#[repr(C)]
#[derive(Clone, Debug, Hash)]
pub struct SessionHandle {
    hdl: u32
}

impl_handle!(SessionHandle);

#[repr(C)]
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum ElucidatorStatus {
    ELUCIDATOR_OK,
    ELUCIDATOR_ERROR,
}

impl ElucidatorStatus {
    pub fn ok() -> Self { Self::ELUCIDATOR_OK }
    pub fn err() -> Self { Self::ELUCIDATOR_ERROR }
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct BoundingBox {
    xmin: f64,
    xmax: f64,
    ymin: f64,
    ymax: f64,
    zmin: f64,
    zmax: f64,
    tmin: f64,
    tmax: f64,
}

/// Instantiate a new Elucidator session. Individual sessions will have
/// different designation to specification relationships.
#[no_mangle]
pub extern "C" fn new_session(sh: &mut *mut SessionHandle, _kind: DatabaseKind) -> ElucidatorStatus {
    let rdb = match RTreeDatabase::new(None, None) {
        Ok(o) => o,
        Err(_) => {
            *sh = ptr::null_mut::<SessionHandle>().to_owned();
            return ElucidatorStatus::err();
        }
    };
    let mut hdl = SessionHandle::get_new();
    SESSION_MAP.write().unwrap()
        .insert(hdl.clone(), rdb);
    *sh = &mut hdl;
    ElucidatorStatus::ok()
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

/// Register the given name and specification to a given session handle.
/// On failure, an error handle will be placed into the provided pointer.
/// Runtime should be O(1) unless the insertion causes a re-hash of a
/// module-level HashMap, which will take O(n) with n the number of
/// designations.
#[no_mangle]
pub extern "C" fn add_spec_to_session(
    name: *const c_char,
    spec: *const c_char,
    sh: *const SessionHandle,
    eh: *mut ErrorHandle,
) -> ElucidatorStatus {
    let name = String::from_utf8_lossy(
        unsafe { CStr::from_ptr(name) }.to_bytes()
    );
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
            return ElucidatorStatus::err();
        },
    };
    unsafe {
        let mut map = SESSION_MAP.write().unwrap();
        let mut session = map.get_mut(&(*sh).clone()).unwrap();
        match &mut session.insert_spec_text(&name, &spec) {
            Ok(_) => ElucidatorStatus::ok(),
            Err(_) => ElucidatorStatus::err(),
        }
    }
}

/// Print a session map
#[no_mangle]
pub extern "C" fn print_session(sh: *const SessionHandle) {
    unsafe {
        let map = SESSION_MAP.read().unwrap();
        assert_eq!(1 as u32, 1 as u32);
        assert_eq!(SessionHandle { hdl: 1 }, SessionHandle { hdl: 1});
        assert_eq!(*sh, SessionHandle { hdl: 1 });
        let ses = map.get(&(*sh));
        println!("{ses:#?}");
    }

    unsafe {
        println!("Got {:#?}", *sh);
    }
}

/// Print it all
#[no_mangle]
pub extern "C" fn print_the_mayhem() {
    println!("{:#?}", SESSION_MAP.read().unwrap());
}
