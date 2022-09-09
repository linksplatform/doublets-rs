use doublets::{
    data::{Flow, LinksConstants},
    Link,
};
use doublets_ffi::{
    constants::Constants,
    errors::{free_error, read_error, DoubletsError},
    export::{doublets_create_log_handle, doublets_free_log_handle},
    logging::{Format, Level},
    store::{constants_from_store, create_unit_store, delete, free_store},
    utils::Fallible,
    FFIContext,
};
use std::{
    ffi::{c_char, CStr, CString},
    fs,
    ptr::null_mut,
};

unsafe extern "C" fn callback(_: FFIContext, ptr: *const c_char) {
    print!("{}", CStr::from_ptr(ptr).to_str().unwrap());
}

extern "C" fn create_cb(_: FFIContext, _: Link<u64>, _: Link<u64>) -> Flow {
    Flow::Continue
}

fn main() {
    unsafe {
        let log_handle =
            doublets_create_log_handle(null_mut(), callback, Level::Trace, Format::Virgin, true);

        let path = CString::new("doublets.links").unwrap();
        let mut handle =
            create_unit_store::<u64>(path.as_ptr(), Constants::from(LinksConstants::external()))
                .unwrap();

        let any = constants_from_store::<u64>(&handle).any;

        let query = [1 /* not exists index */, any, any];
        let result = delete::<u64>(&mut handle, query.as_ptr(), 3, null_mut(), create_cb);

        if let Fallible::Err(error) = result {
            let memchr = |buf: &[u8]| buf.iter().position(|x| *x == 0).unwrap();

            let mut msg_buf = vec![0u8; 256];

            read_error::<u64>(msg_buf.as_mut_ptr().cast(), 256, &error);

            msg_buf.drain(memchr(&msg_buf) + 1..);

            let cstring = CString::from_vec_with_nul(msg_buf).unwrap();
            let str = cstring.to_str().unwrap();
            tracing::error!("{}", str);

            free_error::<u64>(error);
        } else {
            unreachable!()
        }

        free_store::<u64>(handle);

        doublets_free_log_handle(log_handle);
    }
    let _ = fs::remove_file("doublets.links");
}
