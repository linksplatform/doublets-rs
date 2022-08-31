use doublets::{
    data::{Flow, LinksConstants},
    Link,
};
use doublets_ffi::{
    constants::Constants,
    export::{doublets_create_log_handle, doublets_free_log_handle},
    logging::{Format, Level},
    store::{create, doublets_create_united_store_u64, StoreHandle},
    FFICallbackContext,
};
use std::{
    ffi::{c_char, CStr, CString},
    fs,
    ptr::{null, null_mut},
};

unsafe extern "C" fn callback(_: FFICallbackContext, ptr: *const c_char) {
    print!("{}", CStr::from_ptr(ptr).to_str().unwrap());
}

extern "C" fn create_cb<F>(ctx: FFICallbackContext, before: Link<u64>, after: Link<u64>) -> Flow
where
    F: FnMut(Link<u64>, Link<u64>),
{
    unsafe {
        let handler = &mut *(ctx as *mut F);
        (*handler)(before, after);
        Flow::Continue
    }
}

unsafe fn magic_create<F>(handle: &mut StoreHandle<u64>, mut handler: F)
where
    F: FnMut(Link<u64>, Link<u64>),
{
    let ctx = &mut handler as *mut _;
    let _ = create(handle, null(), 0, ctx as *mut _, create_cb::<F>);
}

fn main() {
    unsafe {
        let handle =
            doublets_create_log_handle(null_mut(), callback, Level::Trace, Format::Virgin, false);

        let path = CString::new("doublets.links").unwrap();
        let mut store = doublets_create_united_store_u64(
            path.as_ptr(),
            Constants::from(LinksConstants::external()),
        );

        magic_create(&mut store, |before, after| {
            print!("{before:?}\n{after:?}\n");
        });

        doublets_free_log_handle(handle);
    }
    let _ = fs::remove_file("doublets.links");
}
