use doublets::{data::LinksConstants, Link};
use doublets_ffi::*;
use std::{cell::Cell, ffi::CString};

thread_local! {
    static CALLS: Cell<usize> = const { Cell::new(0) };
}

macro_rules! ffi_round_trip {
    ($test:ident, $ty:ty, $new:ident, $new_with_constants:ident, $drop:ident, $constants:ident, $create:ident,
     $smart_create:ident, $smart_update:ident, $each:ident, $count:ident, $update:ident, $delete:ident) => {
        #[test]
        fn $test() {
            extern "C" fn keep_going(_: Link<$ty>, _: Link<$ty>) -> $ty {
                LinksConstants::<$ty>::external().r#continue
            }
            extern "C" fn stop(_: Link<$ty>, _: Link<$ty>) -> $ty {
                LinksConstants::<$ty>::external().r#break
            }
            extern "C" fn stop_read(_: Link<$ty>) -> $ty {
                CALLS.with(|calls| calls.set(calls.get() + 1));
                LinksConstants::<$ty>::external().r#break
            }
            let dir = tempfile::tempdir().unwrap();
            let path = CString::new(dir.path().join("links").to_str().unwrap()).unwrap();
            // SAFETY: handles are live until dropped, and all slices and callbacks
            // remain valid for their calls. Reopening happens only after dropping.
            unsafe {
                let handle = $new(path.as_ptr());
                assert!(!handle.is_null());
                let constants = $constants(handle);
                let first = $smart_create(handle);
                assert_eq!($smart_update(handle, first, first, first), first);
                assert_eq!(
                    $create(handle, std::ptr::null(), 0, keep_going),
                    constants.r#continue
                );
                assert_eq!(
                    $create(handle, std::ptr::null(), 0, stop),
                    constants.r#break
                );
                assert_eq!($count(handle, std::ptr::null(), 0), 3);
                let query = [2];
                let change = [2, first, first];
                assert_eq!(
                    $update(handle, query.as_ptr(), 1, change.as_ptr(), 3, keep_going),
                    constants.r#continue
                );
                CALLS.with(|calls| calls.set(0));
                assert_eq!(
                    $each(handle, std::ptr::null(), 0, stop_read),
                    constants.r#break
                );
                assert_eq!(CALLS.with(Cell::get), 1);
                assert_eq!(
                    $delete(handle, query.as_ptr(), 1, keep_going),
                    constants.r#continue
                );
                assert_eq!($count(handle, std::ptr::null(), 0), 2);
                $drop(handle);
                let reopened = $new_with_constants(path.as_ptr(), constants);
                assert!(!reopened.is_null());
                assert_eq!($count(reopened, std::ptr::null(), 0), 2);
                $drop(reopened);
            }
        }
    };
}

ffi_round_trip!(
    byte_round_trip,
    u8,
    ByteLinks_New,
    ByteLinks_NewWithConstants,
    ByteLinks_Drop,
    ByteLinks_GetConstants,
    ByteLinks_Create,
    ByteLinks_SmartCreate,
    ByteLinks_SmartUpdate,
    ByteLinks_Each,
    ByteLinks_Count,
    ByteLinks_Update,
    ByteLinks_Delete
);
ffi_round_trip!(
    u16_round_trip,
    u16,
    UInt16Links_New,
    UInt16Links_NewWithConstants,
    UInt16Links_Drop,
    UInt16Links_GetConstants,
    UInt16Links_Create,
    UInt16Links_SmartCreate,
    UInt16Links_SmartUpdate,
    UInt16Links_Each,
    UInt16Links_Count,
    UInt16Links_Update,
    UInt16Links_Delete
);
ffi_round_trip!(
    u32_round_trip,
    u32,
    UInt32Links_New,
    UInt32Links_NewWithConstants,
    UInt32Links_Drop,
    UInt32Links_GetConstants,
    UInt32Links_Create,
    UInt32Links_SmartCreate,
    UInt32Links_SmartUpdate,
    UInt32Links_Each,
    UInt32Links_Count,
    UInt32Links_Update,
    UInt32Links_Delete
);
ffi_round_trip!(
    u64_round_trip,
    u64,
    UInt64Links_New,
    UInt64Links_NewWithConstants,
    UInt64Links_Drop,
    UInt64Links_GetConstants,
    UInt64Links_Create,
    UInt64Links_SmartCreate,
    UInt64Links_SmartUpdate,
    UInt64Links_Each,
    UInt64Links_Count,
    UInt64Links_Update,
    UInt64Links_Delete
);

#[test]
fn constants_round_trip_preserves_optional_external_range() {
    for original in [
        LinksConstants::<u64>::new(),
        LinksConstants::<u64>::external(),
    ] {
        let converted: LinksConstants<u64> = Constants::from(original.clone()).into();
        assert_eq!(converted.internal_range, original.internal_range);
        assert_eq!(converted.external_range, original.external_range);
        assert_eq!(converted.any, original.any);
        assert_eq!(converted.error, original.error);
    }
}

#[test]
fn invalid_path_returns_null() {
    let dir = tempfile::tempdir().unwrap();
    let path = CString::new(dir.path().join("missing/links").to_str().unwrap()).unwrap();
    // SAFETY: path points to a valid, nul-terminated string.
    assert!(unsafe { UInt64Links_New(path.as_ptr()) }.is_null());
}

#[test]
fn error_log() {
    let logger = build_shared_logger();
    setup_shared_logger(logger);
    log::trace!("trace");
    log::debug!("debug");
    log::info!("info");
    log::warn!("warn");
    log::error!("error");
}
