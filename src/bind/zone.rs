use std::{ffi::CStr, ptr::null_mut};

use crate::{
    dns_fixedname_initname, dns_master_style_default, dns_masterformat_t_dns_masterformat_text,
    dns_name_fromtext, dns_rootname, dns_zone_create, dns_zone_detach, dns_zone_load,
    dns_zone_setclass, dns_zone_setdbtype, dns_zone_setmaxttl, dns_zone_setoption,
    dns_zone_setorigin, dns_zone_setstream, dns_zone_settype, dns_zone_t,
    dns_zoneopt_t_DNS_ZONEOPT_CHECKDUPRR, dns_zoneopt_t_DNS_ZONEOPT_CHECKINTEGRITY,
    dns_zoneopt_t_DNS_ZONEOPT_CHECKMX, dns_zoneopt_t_DNS_ZONEOPT_CHECKNAMES,
    dns_zoneopt_t_DNS_ZONEOPT_CHECKNS, dns_zoneopt_t_DNS_ZONEOPT_CHECKSIBLING,
    dns_zoneopt_t_DNS_ZONEOPT_CHECKSPF, dns_zoneopt_t_DNS_ZONEOPT_CHECKSVCB,
    dns_zoneopt_t_DNS_ZONEOPT_CHECKWILDCARD, dns_zoneopt_t_DNS_ZONEOPT_MANYERRORS,
    dns_zoneopt_t_DNS_ZONEOPT_NOMERGE, dns_zoneopt_t_DNS_ZONEOPT_WARNMXCNAME,
    dns_zoneopt_t_DNS_ZONEOPT_WARNSRVCNAME, dns_zonetype_t_dns_zone_primary,
    isc_result_ISC_R_SUCCESS,
};

use crate::bind::mem::IscMem;

use super::{
    buffer::{IscBuffer, MemFile},
    log::IscLog,
};

pub struct DnsZone {
    log: IscLog,
    _mem: IscMem,
    zone: *mut dns_zone_t,
}

impl DnsZone {
    pub fn new() -> Self {
        let mem = IscMem::new();
        let log = IscLog::new(&mem);

        let mut zone: *mut dns_zone_t = std::ptr::null_mut();

        unsafe {
            dns_zone_create(&mut zone, mem.as_ptr(), 0);
        }

        DnsZone {
            _mem: mem,
            zone,
            log,
        }
    }

    pub fn check(&self, zonename: &str, text: &str) -> String {
        unsafe {
            dns_zone_settype(self.zone, dns_zonetype_t_dns_zone_primary);

            let buffer = IscBuffer::from_str(zonename);
            let mut fixorigin = std::mem::zeroed();
            let origin = dns_fixedname_initname(&mut fixorigin);
            let ret = dns_name_fromtext(origin, buffer.as_ptr(), dns_rootname, 0, null_mut());
            if ret != isc_result_ISC_R_SUCCESS {
                panic!("dns_name_fromtext failed");
            }
            let ret = dns_zone_setorigin(self.zone, origin);
            if ret != isc_result_ISC_R_SUCCESS {
                panic!("dns_zone_setorigin failed");
            }

            let dbtype = [CStr::from_bytes_with_nul_unchecked(b"qpzone\0").as_ptr()];
            dns_zone_setdbtype(self.zone, 1, dbtype.as_ptr());

            dns_zone_setclass(self.zone, 1);
            let zone_options = dns_zoneopt_t_DNS_ZONEOPT_CHECKNS
                | dns_zoneopt_t_DNS_ZONEOPT_CHECKMX
                | dns_zoneopt_t_DNS_ZONEOPT_CHECKDUPRR
                | dns_zoneopt_t_DNS_ZONEOPT_CHECKSPF
                | dns_zoneopt_t_DNS_ZONEOPT_MANYERRORS
                | dns_zoneopt_t_DNS_ZONEOPT_CHECKNAMES
                | dns_zoneopt_t_DNS_ZONEOPT_CHECKINTEGRITY
                | dns_zoneopt_t_DNS_ZONEOPT_CHECKSIBLING
                | dns_zoneopt_t_DNS_ZONEOPT_CHECKSVCB
                | dns_zoneopt_t_DNS_ZONEOPT_CHECKWILDCARD
                | dns_zoneopt_t_DNS_ZONEOPT_WARNMXCNAME
                | dns_zoneopt_t_DNS_ZONEOPT_WARNSRVCNAME;
            dns_zone_setoption(self.zone, zone_options, true);
            dns_zone_setoption(self.zone, dns_zoneopt_t_DNS_ZONEOPT_NOMERGE, true);
            dns_zone_setmaxttl(self.zone, 0);

            let input = MemFile::from_str(text);
            let ret = dns_zone_setstream(
                self.zone,
                input.as_ptr(),
                dns_masterformat_t_dns_masterformat_text,
                &dns_master_style_default,
            );
            if ret != isc_result_ISC_R_SUCCESS {
                panic!("dns_zone_setstream failed");
            }
            let ret = dns_zone_load(self.zone, false);
            if ret != isc_result_ISC_R_SUCCESS {
                println!("dns_zone_load failed");
            }

            self.log.to_string()
        }
    }
}

impl Drop for DnsZone {
    fn drop(&mut self) {
        unsafe {
            dns_zone_detach(&mut self.zone);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_zone_check() {
        let zonename = "example.com";
        let text = r#"
$TTL 86400
@   IN  SOA ns1.example.com. admin.example.com. (
            2023101001 ; Serial
            3600       ; Refresh
            1800       ; Retry
            604800     ; Expire
            86400      ; Minimum TTL
)
    IN  NS  ns1.example.com.
    IN  NS  ns2.example.com.

ns1 IN  A 192.168.1.1
ns2 IN  A 192.168.1.2
"#;

        let zone = DnsZone::new();
        let result = zone.check(zonename, &text);
        println!("{}", result);
    }
}
