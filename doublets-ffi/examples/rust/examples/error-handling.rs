use doublets::{
    data::{Flow, LinksConstants},
    Link,
};
use doublets_ffi::{
    constants::Constants,
    errors::{
        doublets_read_error, doublets_read_error_as_not_found, doublets_read_error_message,
        DoubletsResultKind,
    },
    export::{doublets_create_log_handle, doublets_free_log_handle},
    store::{constants_for_store, create_unit_store, delete, free_store},
    FFICallbackContext,
};
use std::{
    ffi::{c_char, CStr, CString},
    fs,
    ptr::null_mut,
};

unsafe extern "C" fn callback(_: FFICallbackContext, ptr: *const c_char) {
    print!("{}", CStr::from_ptr(ptr).to_str().unwrap());
}

extern "C" fn create_cb(_: FFICallbackContext, _: Link<u64>, _: Link<u64>) -> Flow {
    Flow::Continue
}

fn main() {
    let level = CString::new("trace").unwrap();
    unsafe {
        let handle = doublets_create_log_handle(null_mut(), callback, level.as_ptr(), true, false);

        let path = CString::new("doublets.links").unwrap();
        let mut store =
            create_unit_store::<u64>(path.as_ptr(), Constants::from(LinksConstants::external()));

        // `StoreHandle` is transparent - in really FFI we must use raw ptr
        if store.as_ptr().is_null() {
            unreachable!("it would be better for errors not to occur in the examples")
        }

        let ptr = store.assume() as *mut _ as *mut _;

        let any = constants_for_store::<u64>(ptr).any;

        let query = [1 /* not exists index */, any, any];
        let result = delete::<u64>(ptr, query.as_ptr(), 3, null_mut(), create_cb);

        if result as u8 >= DoubletsResultKind::NotExists as u8 {
            let memchr = |buf: &[u8]| buf.iter().position(|x| *x == 0).unwrap();

            // last error - DON'T USE PTR AFTER NEW DOUBLETS OPERATION
            let err = doublets_read_error();
            let mut msg_buf = vec![0u8; 256];

            doublets_read_error_message(msg_buf.as_mut_ptr().cast(), 256, err);

            msg_buf.drain(memchr(&msg_buf) + 1..);

            let cstring = CString::from_vec_with_nul(msg_buf).unwrap();
            let str = cstring.to_str().unwrap();
            tracing::error!("{}", str);

            // forget `err` ptr - we not in manage it deallocation
            let not_exists = doublets_read_error_as_not_found(err);
            tracing::error!("duplication: link {} does not exists", not_exists);

            // forget `err` ptr - we not in manage it deallocation
        }

        free_store::<u64>(ptr);

        doublets_free_log_handle(handle);
    }
    let _ = fs::remove_file("doublets.links");
}
