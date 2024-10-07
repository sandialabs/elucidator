use elucidator::error::ElucidatorError;

use elucidator_db::{
    backends::rtree::RTreeDatabase,
    database::{Database, Metadata},
    error,
};

use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    fmt,
    hash::Hash,
    mem,
    os::raw::c_char,
    ptr, slice,
    sync::{
        atomic::{AtomicU32, Ordering},
        LazyLock, RwLock,
    },
};

type Emap = LazyLock<RwLock<HashMap<ErrorHandle, ApiError>>>;
static ERROR_MAP: Emap = LazyLock::new(|| RwLock::new(HashMap::new()));

type Smap = LazyLock<RwLock<HashMap<SessionHandle, RTreeDatabase>>>;
static SESSION_MAP: Smap = LazyLock::new(|| RwLock::new(HashMap::new()));

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
    fn htype() -> String;
}

macro_rules! impl_handle {
    ($($tt:ty), *) => {
        $(
            impl Eq for $tt {}
            impl Handle for $tt {
                fn get_new() -> Self {
                    let hdl = HANDLE_NUM.fetch_add(1, Ordering::SeqCst);
                    Self { hdl: hdl.clone() }
                }
                fn id(&self) -> u32 { self.hdl }
                fn htype() -> String {
                    stringify!($tt).to_string()
                }
            }
        )*
    };
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct ErrorHandle {
    hdl: u32,
}

impl_handle!(ErrorHandle);

#[repr(C)]
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct SessionHandle {
    hdl: u32,
}

#[derive(Debug)]
enum ApiError {
    Eluci(ElucidatorError),
    Database(error::DatabaseError),
    HandleNotFound {
        address: String,
        id: u32,
        handle_type: String,
    },
    #[allow(dead_code)]
    DesignationNotFound {
        session: u32,
        designation: String,
    },
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eluci(e) => {
                write!(f, "ElucidatorError: {e}")
            }
            Self::Database(e) => {
                write!(f, "Elucidator Database Error: {e}")
            }
            Self::HandleNotFound {
                address,
                id,
                handle_type,
            } => {
                write!(
                    f,
                    "Handle {id} not found: type {handle_type} at address {address}"
                )
            }
            Self::DesignationNotFound {
                session,
                designation,
            } => {
                write!(
                    f,
                    "Cannot find designation {designation} in session {session}"
                )
            }
        }
    }
}

impl From<ElucidatorError> for ApiError {
    fn from(error: ElucidatorError) -> Self {
        Self::Eluci(error)
    }
}

impl From<error::DatabaseError> for ApiError {
    fn from(error: error::DatabaseError) -> Self {
        Self::Database(error)
    }
}

fn not_found_from<T: Handle>(hdl: &T) -> ApiError {
    ApiError::HandleNotFound {
        address: format!("{:#?}", ptr::addr_of!(hdl)),
        id: hdl.id(),
        handle_type: T::htype(),
    }
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
    pub fn ok() -> Self {
        Self::ELUCIDATOR_OK
    }
    pub fn err() -> Self {
        Self::ELUCIDATOR_ERROR
    }
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
        *ptr = BufNode { p, n, next };
        ptr
    }
}

unsafe fn blobs_into_bufnode(blobs: &mut Vec<&Vec<u8>>) -> *mut BufNode {
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
pub extern "C" fn free_bufnodes(bf: *mut BufNode) {
    unsafe {
        let mut current = bf;
        while !current.is_null() {
            let next = (*current).next;
            libc::free((*current).p as *mut libc::c_void);
            libc::free(current as *mut libc::c_void);
            current = next;
        }
    }
}

#[no_mangle]
pub extern "C" fn fetch_sample_blob() -> *mut BufNode {
    let a = vec![1, 2, 3, 4, 5];
    let b = vec![2, 3, 5, 7, 11, 13];
    let c = vec![0, 27, 6];

    let mut sample: Vec<&Vec<u8>> = vec![&a, &b, &c];
    unsafe { blobs_into_bufnode(&mut sample) }
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
    SESSION_MAP.write().unwrap().insert(hdl.clone(), rdb);
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
            Some(e) => CString::new(format!("{e}").as_str()).unwrap().into_raw(),
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
    let name = String::from_utf8_lossy(unsafe { CStr::from_ptr(name) }.to_bytes());
    let spec = String::from_utf8_lossy(unsafe { CStr::from_ptr(spec) }.to_bytes());
    let mut map = SESSION_MAP.write().unwrap();
    let hdl = unsafe { (*sh).clone() };
    let session = match map.get_mut(&hdl) {
        Some(ses) => ses,
        None => {
            let ehdl = ErrorHandle::get_new();
            unsafe {
                *eh = ehdl.clone();
            }
            ERROR_MAP
                .write()
                .unwrap()
                .insert(ehdl.clone(), not_found_from(&hdl));
            return ElucidatorStatus::err();
        }
    };
    match &mut session.insert_spec_text(&name, &spec) {
        Ok(_) => ElucidatorStatus::ok(),
        Err(e) => {
            let ehdl = ErrorHandle::get_new();
            unsafe {
                *eh = ehdl.clone();
            }
            ERROR_MAP
                .write()
                .unwrap()
                .insert(ehdl.clone(), (*e).clone().into());
            ElucidatorStatus::err()
        }
    }
}

/// Insert metadata into a session.
#[no_mangle]
pub extern "C" fn insert_metadata_in_session(
    sh: *const SessionHandle,
    bb: BoundingBox,
    designation: *const c_char,
    blob: *const u8,
    n_bytes: usize,
    eh: *mut ErrorHandle,
) -> ElucidatorStatus {
    let designation = String::from_utf8_lossy(unsafe { CStr::from_ptr(designation) }.to_bytes());
    let mut map = SESSION_MAP.write().unwrap();
    let hdl = unsafe { (*sh).clone() };
    let session = match map.get_mut(&hdl) {
        Some(ses) => ses,
        None => {
            let ehdl = ErrorHandle::get_new();
            unsafe {
                *eh = ehdl.clone();
            }
            ERROR_MAP
                .write()
                .unwrap()
                .insert(ehdl.clone(), not_found_from(&hdl));
            return ElucidatorStatus::err();
        }
    };
    let buffer = unsafe { slice::from_raw_parts(blob, n_bytes) };
    let datum = Metadata {
        xmin: bb.a.x,
        xmax: bb.b.x,
        ymin: bb.a.y,
        ymax: bb.b.y,
        zmin: bb.a.z,
        zmax: bb.b.z,
        tmin: bb.a.t,
        tmax: bb.b.t,
        designation: &designation,
        buffer,
    };
    match session.insert_metadata(&datum) {
        Ok(_) => ElucidatorStatus::ok(),
        Err(e) => {
            let ehdl = ErrorHandle::get_new();
            unsafe {
                *eh = ehdl.clone();
            }
            ERROR_MAP
                .write()
                .unwrap()
                .insert(ehdl.clone(), ApiError::Database(e.clone()));
            ElucidatorStatus::err()
        }
    }
}

/// Get metadata overlapping a point
#[no_mangle]
pub extern "C" fn get_metadata_in_bb(
    sh: *const SessionHandle,
    bb: BoundingBox,
    designation: *const c_char,
    epsilon: f64,
    results: *mut *mut BufNode,
    eh: *mut ErrorHandle,
) -> ElucidatorStatus {
    let designation = String::from_utf8_lossy(unsafe { CStr::from_ptr(designation) }.to_bytes());
    let mut map = SESSION_MAP.write().unwrap();
    let hdl = unsafe { (*sh).clone() };
    let session = match map.get_mut(&hdl) {
        Some(ses) => ses,
        None => {
            let ehdl = ErrorHandle::get_new();
            unsafe {
                *eh = ehdl.clone();
            }
            ERROR_MAP
                .write()
                .unwrap()
                .insert(ehdl.clone(), not_found_from(&hdl));
            return ElucidatorStatus::err();
        }
    };
    let mut r = session.get_metadata_blobs_in_bb(
        bb.a.x,
        bb.b.x,
        bb.a.y,
        bb.b.y,
        bb.a.z,
        bb.b.z,
        bb.a.t,
        bb.b.t,
        &designation,
        Some(epsilon),
    );
    match &mut r {
        Ok(o) => {
            unsafe {
                let bn = blobs_into_bufnode(o);
                *results = bn;
            }
            ElucidatorStatus::ok()
        }
        Err(e) => {
            let ehdl = ErrorHandle::get_new();
            unsafe {
                *eh = ehdl.clone();
            }
            ERROR_MAP
                .write()
                .unwrap()
                .insert(ehdl.clone(), ApiError::Database(e.clone()));
            ElucidatorStatus::err()
        }
    }
}

/// Print a session map
#[no_mangle]
pub extern "C" fn print_session(sh: *const SessionHandle) {
    unsafe {
        let map = SESSION_MAP.read().unwrap();
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
