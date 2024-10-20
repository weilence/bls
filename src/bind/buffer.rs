use std::{os::raw::c_void, ptr::null_mut};

use bind_parser::{isc_buffer__bindgen_ty_1, isc_buffer_t, ISC_BUFFER_MAGIC};

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
    pub fn new(str: &str) -> Self {
        let isc_buffer = Box::into_raw(Box::new(isc_buffer_t {
            base: str.as_ptr() as *mut c_void,
            length: str.len() as u32,
            used: 0,
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

    pub fn add(&mut self, length: u32) {
        let v = unsafe { &mut *self.isc_buffer };
        if v.used + length > v.length {
            panic!("buffer overflow");
        }

        v.used += length;
    }

    pub fn as_ptr(&self) -> *mut isc_buffer_t {
        self.isc_buffer
    }
}
