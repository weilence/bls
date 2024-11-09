use std::{
    ffi::{CStr, CString},
    fmt::Display,
    os::raw::c_void,
    ptr::null_mut,
};

use crate::{fclose, fmemopen, isc_buffer__bindgen_ty_1, isc_buffer_t, FILE, ISC_BUFFER_MAGIC};

pub struct IscBuffer {
    isc_buffer: *mut isc_buffer_t,
}

impl Drop for IscBuffer {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.isc_buffer);
        };
    }
}

impl IscBuffer {
    pub fn from_str(str: &str) -> Self {
        let isc_buffer = Box::into_raw(Box::new(isc_buffer_t {
            base: str.as_ptr() as *mut c_void,
            length: str.len() as u32,
            used: str.len() as u32,
            link: isc_buffer__bindgen_ty_1 {
                prev: -1_isize as *mut isc_buffer_t,
                next: -1_isize as *mut isc_buffer_t,
            },
            magic: ISC_BUFFER_MAGIC,
            current: 0,
            active: 0,
            extra: 0,
            dynamic: false,
            mctx: null_mut(),
        }));

        IscBuffer { isc_buffer }
    }

    pub fn as_ptr(&self) -> *mut isc_buffer_t {
        self.isc_buffer
    }
}

pub struct MemFile {
    file: *mut FILE,
    ptr: *mut c_void,
}

impl MemFile {
    pub fn new<const N: usize>() -> Self {
        let ptr = Box::into_raw(Box::new([0u8; N])) as *mut c_void;
        let file = unsafe { fmemopen(ptr, N, "w+\0".as_ptr() as *const i8) };

        MemFile { file, ptr }
    }

    pub fn from_str(s: &str) -> Self {
        let cstring: CString = CString::new(s).expect("failed to convert str to CString");
        let len = cstring.count_bytes();
        let ptr = cstring.into_raw() as *mut c_void;

        let file = unsafe { fmemopen(ptr, len, "r\0".as_ptr() as *const i8) };

        MemFile { file, ptr }
    }

    pub fn as_ptr(&self) -> *mut FILE {
        self.file
    }
}

impl Display for MemFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = unsafe { CStr::from_ptr(self.ptr as *const i8) };
        let str = s.to_str().expect("failed to convert CStr to str");
        write!(f, "{}", str)
    }
}

impl Drop for MemFile {
    fn drop(&mut self) {
        unsafe {
            fclose(self.file);
            let _ = Box::from_raw(self.ptr);
        };
    }
}
