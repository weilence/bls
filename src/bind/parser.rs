use std::{
    error::Error,
    ffi::CString,
    os::raw::c_char,
    path::Path,
    ptr::{addr_of, null_mut},
};

use crate::{
    cfg_parse_buffer, cfg_parse_file, cfg_parser_create, cfg_parser_destroy, cfg_parser_reset,
    cfg_parser_t, cfg_type_namedconf, isc_result_ISC_R_SUCCESS,
};

use super::{buffer::IscBuffer, log::IscLog, mem::IscMem, obj::Obj};

pub struct IscParser {
    isc_parser: *mut cfg_parser_t,
}

impl IscParser {
    pub fn new(mem: &IscMem, log: &IscLog) -> Result<Self, Box<dyn Error>> {
        let mut isc_parser = null_mut();
        let ret = unsafe { cfg_parser_create(mem.as_ptr(), log.as_ptr(), &mut isc_parser) };
        if ret != isc_result_ISC_R_SUCCESS {
            return Err("cfg_parser_create failed".into());
        }

        Ok(IscParser { isc_parser })
    }

    pub fn parse_file(&self, path: &Path) -> Result<Obj, Box<dyn Error>> {
        let file = CString::new(path.to_str().unwrap()).unwrap();

        let mut obj = null_mut();
        let ret = unsafe {
            cfg_parser_reset(self.isc_parser);

            cfg_parse_file(
                self.isc_parser,
                file.as_ptr(),
                addr_of!(cfg_type_namedconf),
                &mut obj,
            )
        };
        if ret != isc_result_ISC_R_SUCCESS {
            return Err("cfg_parse_file failed".into());
        }

        Ok(Obj::new(self, obj))
    }

    #[allow(dead_code)]
    pub fn parse_string(&self, str: &str) -> Result<Obj, Box<dyn Error>> {
        let mut conf = null_mut();

        let isc_buffer = IscBuffer::from_str(str);

        let ret = unsafe {
            cfg_parser_reset(self.isc_parser);

            cfg_parse_buffer(
                self.isc_parser,
                isc_buffer.as_ptr(),
                "named.conf\0".as_ptr() as *const c_char,
                0,
                addr_of!(cfg_type_namedconf),
                0,
                &mut conf,
            )
        };
        if ret != isc_result_ISC_R_SUCCESS {
            return Err("cfg_parse_buffer failed".into());
        }

        Ok(Obj::new(self, conf))
    }

    pub fn as_ref(&self) -> *mut cfg_parser_t {
        self.isc_parser
    }
}

impl Drop for IscParser {
    fn drop(&mut self) {
        unsafe {
            cfg_parser_destroy(&mut self.isc_parser);
        }
    }
}

#[test]
fn test_parser() {
    let mem = IscMem::new();
    let log = IscLog::new(&mem);

    let parser = IscParser::new(&mem, &log).unwrap();

    let obj = parser
        .parse_string("options\n{\nrecursion yes;\n};\n")
        .unwrap();

    if obj.check(&log, &mem) {
        print!("{}", log);
    }
}
