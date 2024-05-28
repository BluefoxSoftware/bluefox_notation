use std::{ffi::CString, intrinsics::size_of, ptr::{null, null_mut}};

use libc::{ c_char, c_double, c_long, c_void };
use super::{BluefoxData, BluefoxDataType};

#[repr(C)]
pub enum CBluefoxDataTypes {
    NULL = 0,
    BOOL = 1,
    INT = 2,
    FLOAT = 3,
    STRING = 4,
    FUNCTION = 5,
    ARRAY = 6,
    DATA = 7
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CBluefoxArray {
    l: c_long,
    d: *const CBluefoxDataType
}

#[no_mangle]
pub unsafe fn bluefox_new_array() -> CBluefoxArray {
    CBluefoxArray {
        l: 0,
        d: null()
    }
}

#[no_mangle]
pub unsafe fn bluefox_array_push(a: *mut CBluefoxArray, v: CBluefoxDataType) {
    let l = (*a).l;
    let od = (*a).d;

    let d = libc::malloc(size_of::<CBluefoxDataType>() * (l as usize + 1)) as *mut CBluefoxDataType;

    if !od.is_null() {
        libc::memcpy(d as *mut c_void, od as *const c_void, l as usize);
    }

    *d.offset(l as isize) = v.clone();
    *a = CBluefoxArray {
        l: l + 1,
        d
    }
}

#[no_mangle]
pub unsafe fn bluefox_array_get(a: *const CBluefoxArray, idx: c_long) -> *const CBluefoxDataType {
    if idx < (*a).l {
        return (*a).d.offset(idx as isize);
    }
    null()
}

unsafe fn to_cstring(s: String) -> *const c_char {
    let ns = libc::malloc(size_of::<c_char>() * s.len()) as *mut c_char; // claim a spot in memory
    let rs = CString::new(s.clone()).unwrap().as_ptr(); // creates temporary value
    libc::memcpy(ns as *mut c_void, rs as *const c_void, s.len()); // value is copied to more permanent location
    ns
    // temporary value dropped, its oki because it has been copied
}
unsafe fn to_string(ptr: *const u8) -> String {
    let mut len: usize = 0;
    loop {
        let value = *(ptr.offset(len as isize));
        len += 1;
        if value == 0 {
            break;
        }
    }
    let slice = std::slice::from_raw_parts(ptr, len);
    String::from_utf8_unchecked(slice.to_vec())
}
unsafe fn to_array(ptr: *const c_void) -> Vec<BluefoxDataType<'static>> {
    let value = *(ptr as *const CBluefoxArray);
    let slice = std::slice::from_raw_parts(value.d as *const CBluefoxDataType, value.l as usize);
    let mut output = vec![];
    for v in slice {
        output.push(BluefoxDataType::from(*v));
    }
    output
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CBluefoxDataType {
    t: c_long,
    v: *const c_void
}
impl From<CBluefoxDataType> for BluefoxDataType {
    fn from(value: CBluefoxDataType) -> Self {
        unsafe {
            match value.t {
                0 => BluefoxDataType::NULL,
                1 => BluefoxDataType::BOOL(if *(value.v as *const c_long) == 0 { false } else { true }),
                2 => BluefoxDataType::INT(*(value.v as *const c_long) as i64),
                3 => BluefoxDataType::FLOAT(*(value.v as *const c_double) as f64),
                4 => BluefoxDataType::STRING(to_string(value.v as *const u8)),
                5 => BluefoxDataType::FUNCTION(to_string(value.v as *const u8)),
                6 => BluefoxDataType::ARRAY(to_array(value.v)),
                7 => BluefoxDataType::DATA(BluefoxData::from(*(value.v as *const CBluefoxData))),
                _ => BluefoxDataType::NULL,
            }
        }
    }
} 
impl From<BluefoxDataType> for CBluefoxDataType {
    fn from(value: BluefoxDataType) -> Self {
        unsafe {
            match value {
                BluefoxDataType::NULL => bluefox_new_null_data(),
                BluefoxDataType::BOOL(x) => bluefox_new_bool_data(if x { 1 } else { 0 }),
                BluefoxDataType::INT(x) => bluefox_new_int_data(x),
                BluefoxDataType::FLOAT(x) => bluefox_new_float_data(x),
                BluefoxDataType::STRING(x) => bluefox_new_string_data(to_cstring(x)),
                BluefoxDataType::FUNCTION(x) => bluefox_new_function_data(to_cstring(x)),
                BluefoxDataType::ARRAY(x) => {
                    let mut output = bluefox_new_array();
                    for data in x {
                        bluefox_array_push(&mut output, CBluefoxDataType::from(data));
                    }
                    bluefox_new_array_data(output)
                },
                BluefoxDataType::DATA(x) => bluefox_new_data_data(CBluefoxData::from(x))
            }
        }
    }
}

#[no_mangle]
pub unsafe fn bluefox_new_null_data() -> CBluefoxDataType {
    CBluefoxDataType {
        t: 0,
        v: null()
    }
}

#[no_mangle]
pub unsafe fn bluefox_new_bool_data(b: c_long) -> CBluefoxDataType {
    let v = libc::malloc(size_of::<c_long>()) as *mut c_long;
    *v = b.clone();
    CBluefoxDataType {
        t: 1,
        v: v as *const c_void
    }
}

#[no_mangle]
pub unsafe fn bluefox_new_int_data(i: c_long) -> CBluefoxDataType {
    let v = libc::malloc(size_of::<c_long>()) as *mut c_long;
    *v = i.clone();
    CBluefoxDataType {
        t: 2,
        v: v as *const c_void
    }
}

#[no_mangle]
pub unsafe fn bluefox_new_float_data(f: c_double) -> CBluefoxDataType {
    let v = libc::malloc(size_of::<c_double>()) as *mut c_double;
    *v = f.clone();
    CBluefoxDataType {
        t: 3,
        v: v as *const c_void
    }
}

#[no_mangle]
pub unsafe fn bluefox_new_string_data(s: *const c_char) -> CBluefoxDataType {
    CBluefoxDataType {
        t: 4,
        v: s as *const c_void
    }
}

#[no_mangle]
pub unsafe fn bluefox_new_function_data(s: *const c_char) -> CBluefoxDataType {
    CBluefoxDataType {
        t: 5,
        v: s as *const c_void
    }
}

#[no_mangle]
pub unsafe fn bluefox_new_array_data(a: CBluefoxArray) -> CBluefoxDataType {
    let v = libc::malloc(size_of::<CBluefoxArray>()) as *mut CBluefoxArray;
    *v = a.clone();
    CBluefoxDataType {
        t: 6,
        v: v as *const c_void
    }
}

#[no_mangle]
pub unsafe fn bluefox_new_data_data(d: CBluefoxData) -> CBluefoxDataType {
    let v = libc::malloc(size_of::<CBluefoxData>()) as *mut CBluefoxData;
    *v = d.clone();
    CBluefoxDataType {
        t: 7,
        v: v as *const c_void
    }
}

#[no_mangle]
pub unsafe fn bluefox_data_is_null(d: *const CBluefoxDataType) -> c_long {
    if (*d).t == 0 {
        return 1;
    }
    0
}

#[no_mangle]
pub unsafe fn bluefox_data_get_bool(d: *const CBluefoxDataType) -> *const c_long {
    if (*d).t == 1 {
        return (*d).v as *const c_long;
    }
    null()
}

#[no_mangle]
pub unsafe fn bluefox_data_get_int(d: *const CBluefoxDataType) -> *const c_long {
    if (*d).t == 2 {
        return (*d).v as *const c_long;
    }
    null()
}

#[no_mangle]
pub unsafe fn bluefox_data_get_float(d: *const CBluefoxDataType) -> *const c_double {
    if (*d).t == 3 {
        return (*d).v as *const c_double;
    }
    null()
}

#[no_mangle]
pub unsafe fn bluefox_data_get_string(d: *const CBluefoxDataType) -> *const c_char {
    if (*d).t == 4 {
        return (*d).v as *const c_char;
    }
    null()
}

#[no_mangle]
pub unsafe fn bluefox_data_get_function(d: *const CBluefoxDataType) -> *const c_char {
    if (*d).t == 5 {
        return (*d).v as *const c_char;
    }
    null()
}

#[no_mangle]
pub unsafe fn bluefox_data_get_array(d: *const CBluefoxDataType) -> *const CBluefoxArray {
    if (*d).t == 6 {
        return (*d).v as *const CBluefoxArray;
    }
    null()
}

#[no_mangle]
pub unsafe fn bluefox_data_get_data(d: *const CBluefoxDataType) -> *const CBluefoxData {
    if (*d).t == 7 {
        return (*d).v as *const CBluefoxData;
    }
    null()
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CBluefoxData {
    l: c_long,
    k: *const *const c_char,
    v: *const CBluefoxDataType
}
impl From<CBluefoxData> for BluefoxData {
    fn from(value: CBluefoxData) -> Self {
        let mut output = Self::new();
        for i in 0..value.l {
            unsafe { output.data.insert(to_string(*value.k.offset(i as isize) as *const u8), BluefoxDataType::from(*value.v)); }
        }
        output
    }
}
impl From<BluefoxData> for CBluefoxData {
    fn from(value: BluefoxData) -> Self {
        let mut output = bluefox_new_internal_data();
        for (k, v) in value.data {
            unsafe { bluefox_data_insert(&mut output, to_cstring(k), CBluefoxDataType::from(v)); }
        }
        output
    }
}

#[no_mangle]
pub unsafe fn bluefox_new_data() -> *const CBluefoxData {
    let o = libc::malloc(size_of::<CBluefoxData>()) as *mut CBluefoxData;
    *o = bluefox_new_internal_data();
    o as *const CBluefoxData
}

fn bluefox_new_internal_data() -> CBluefoxData {
    CBluefoxData {
        l: 0,
        k: null(),
        v: null()
    }
}

#[no_mangle]
pub unsafe fn bluefox_data_insert(d: *mut CBluefoxData, k: *const c_char, v: CBluefoxDataType) {
    let pv = bluefox_data_get(d, k) as *mut CBluefoxDataType;
    if pv != null_mut() {
        *pv = v.clone(); // if value exists, assign
        return;
    }

    let l = (*d).l;
    let ok = (*d).k;
    let ov = (*d).v;

    let nk = libc::malloc(size_of::<*const c_char>() * (l as usize + 1)) as *mut *const c_char;
    let nv = libc::malloc(size_of::<CBluefoxDataType>() * (l as usize + 1)) as *mut CBluefoxDataType;

    if l > 0 {
        libc::memcpy(nk as *mut c_void, ok as *const c_void, l as usize);
        libc::memcpy(nv as *mut c_void, ov as *const c_void, l as usize);
    }

    *nk.offset(l as isize) = k;
    *nv.offset(l as isize) = v;
    *d = CBluefoxData {
        l: l + 1,
        k: nk,
        v: nv
    }
}

#[no_mangle]
pub unsafe fn bluefox_data_get(d: *const CBluefoxData, k: *const c_char) -> *const CBluefoxDataType {
    for i in 0..(*d).l {
        if libc::strcmp(k, *(*d).k.offset(i as isize)) == 0 {
            return (*d).v.offset(i as isize);
        }
    }
    null()
}

#[no_mangle]
pub unsafe fn bluefox_destroy_type(t: *const CBluefoxDataType) {
    match (*t).t {
        6 => bluefox_destroy_array(bluefox_data_get_array(t) as *mut CBluefoxArray),
        7 => bluefox_destroy_data(bluefox_data_get_data(t) as *mut CBluefoxData),
        _ => libc::free((*t).v as *mut c_void)
    }
}

#[no_mangle]
pub unsafe fn bluefox_destroy_array(a: *const CBluefoxArray) {
    libc::free((*a).d as *mut c_void);
}

#[no_mangle]
pub unsafe fn bluefox_destroy_data(d: *mut CBluefoxData) {
    for i in 0..(*d).l {
        libc::free((*(*d).k.offset(i as isize)) as *mut c_void);
        bluefox_destroy_type((*d).v.offset(i as isize));
    }
    libc::free((*d).k as *mut c_void);
    libc::free((*d).v as *mut c_void);
    libc::free(d as *mut c_void);
}

