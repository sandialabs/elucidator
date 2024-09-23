use elucidator::{
    designation::DesignationSpecification,
    error::ElucidatorError,
};

use elucidator_db::{
    error,
    backends::{sqlite::SqlDatabase, rtree::RTreeDatabase},
    database::Database,
};

use libc;

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    ffi::{CStr, CString},
    mem,
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
pub struct Point {
    x: f64,
    y: f64,
    z: f64,
    t: f64,
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct BoundingBox {
    a: Point,
    b: Point,
}

#[repr(C)]
#[derive(Debug)]
pub struct BufNode {
    p: *mut u8,
    n: usize,
    next: *mut BufNode,
}

impl BufNode {
    unsafe fn from(p: *mut u8, n: usize, next: *mut BufNode) -> *mut Self {
        let ptr = libc::malloc(mem::size_of::<Self>()) as *mut BufNode;
        *ptr = BufNode { p, n, next };
        ptr
    }
    unsafe fn empty() -> *mut BufNode {
        let p = ptr::null_mut::<u8>();
        let n = 0;
        let next = ptr::null_mut::<BufNode>();
        let ptr = libc::malloc(mem::size_of::<Self>()) as *mut BufNode;
        *ptr = BufNode { p, n, next};
        ptr
    }
}

unsafe fn blobs_into_bufnode(blobs: &mut Vec<Vec<u8>>) -> *mut BufNode {
    let mut prev: *mut BufNode = std::ptr::null_mut::<BufNode>();
    let mut bf = BufNode::empty();
    for blob in blobs.iter().rev() {
        let n = blob.len();
        let p = libc::malloc(n) as *mut u8;
        for (i, byte) in blob.iter().enumerate() {
            *(p.wrapping_add(i)) = *byte;
        }
        let next = prev;
        bf = BufNode::from(p, n, next);
        prev = bf;
    }
    blobs.truncate(0);
    bf
}

#[no_mangle]
pub extern "C" fn fetch_sample_blob() -> *mut BufNode {
    let mut sample: Vec<Vec<u8>> = vec![
        vec![1, 2, 3, 4, 5],
        vec![2, 3, 5, 7, 11, 13],
        vec![0, 27, 6],
    ];
    unsafe {
        blobs_into_bufnode(&mut sample)
    }
}

/// Instantiate a new Elucidator session. Individual sessions will have
/// different designation to specification relationships. You must check the
/// return status. If the status is not ELUCIDATOR_OK, an error has occurred
/// and the value of the passed pointer has not been updated.
#[no_mangle]
pub extern "C" fn new_session(sh: *mut SessionHandle, _kind: DatabaseKind) -> ElucidatorStatus {
    let rdb = match RTreeDatabase::new(None, None) {
        Ok(o) => o,
        Err(_) => {
            return ElucidatorStatus::err();
        }
    };
    let hdl = SessionHandle::get_new();
    SESSION_MAP.write().unwrap()
        .insert(hdl.clone(), rdb);
    unsafe {
        *sh = hdl;
    }
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
        let oops = SessionHandle { hdl: 1 };
        assert_eq!((*sh).id(), SessionHandle { hdl: 1 }.id());
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
