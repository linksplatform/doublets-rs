// C ABI specializations share the same implementation for every integer width.
macro_rules! export_links {
    ($ty:ty, $new_united_links:ident, $new_with_constants_united_links:ident, $drop_united_links:ident, $get_constants_united_links:ident, $create_united:ident, $smart_create_united:ident, $smart_update_united:ident, $each_united:ident, $count_united:ident, $update_united:ident, $delete_united:ident) => {
        /// C ABI export for this integer specialization.
        ///
        /// # Safety
        /// `path` must point to a valid NUL-terminated UTF-8 string.
        #[no_mangle]
        pub unsafe extern "C" fn $new_united_links(path: *const c_char) -> *mut c_void {
            new_united_links::<$ty>(path)
        }
        /// C ABI export for this integer specialization.
        ///
        /// # Safety
        /// `path` must point to a valid NUL-terminated UTF-8 string; constants must describe a valid store.
        #[no_mangle]
        pub unsafe extern "C" fn $new_with_constants_united_links(
            path: *const c_char,
            constants: Constants<$ty>,
        ) -> *mut c_void {
            new_with_constants_united_links::<$ty>(path, constants)
        }
        /// C ABI export for this integer specialization.
        ///
        /// # Safety
        /// `this` must be null or a live handle of this integer width, dropped exactly once.
        #[no_mangle]
        pub unsafe extern "C" fn $drop_united_links(this: *mut c_void) {
            drop_united_links::<$ty>(this)
        }
        /// C ABI export for this integer specialization.
        ///
        /// # Safety
        /// `this` must be a live handle of this integer width, with exclusive access for this call.
        #[no_mangle]
        pub unsafe extern "C" fn $get_constants_united_links(this: *mut c_void) -> Constants<$ty> {
            get_constants_united_links::<$ty>(this)
        }
        /// C ABI export for this integer specialization.
        ///
        /// # Safety
        /// `this` must be a live handle of this integer width with exclusive access. Query pointers must be null or valid for their lengths; callbacks must be valid and must not reenter the handle or unwind.
        #[no_mangle]
        pub unsafe extern "C" fn $create_united(
            this: *mut c_void,
            query: *const $ty,
            len: usize,
            callback: CUDCallback<$ty>,
        ) -> $ty {
            create_united::<$ty>(this, query, len, callback)
        }
        /// C ABI export for this integer specialization.
        ///
        /// # Safety
        /// `this` must be a live handle of this integer width with exclusive access. Query pointers must be null or valid for their lengths; callbacks must be valid and must not reenter the handle or unwind.
        #[no_mangle]
        pub unsafe extern "C" fn $smart_create_united(this: *mut c_void) -> $ty {
            smart_create_united::<$ty>(this)
        }
        /// C ABI export for this integer specialization.
        ///
        /// # Safety
        /// `this` must be a live handle of this integer width with exclusive access. Query pointers must be null or valid for their lengths; callbacks must be valid and must not reenter the handle or unwind.
        #[no_mangle]
        pub unsafe extern "C" fn $smart_update_united(
            this: *mut c_void,
            index: $ty,
            source: $ty,
            target: $ty,
        ) -> $ty {
            smart_update_united::<$ty>(this, index, source, target)
        }
        /// C ABI export for this integer specialization.
        ///
        /// # Safety
        /// `this` must be a live handle of this integer width with exclusive access. Query pointers must be null or valid for their lengths; callbacks must be valid and must not reenter the handle or unwind.
        #[no_mangle]
        pub unsafe extern "C" fn $each_united(
            this: *mut c_void,
            query: *const $ty,
            len: usize,
            callback: EachCallback<$ty>,
        ) -> $ty {
            each_united::<$ty>(this, query, len, callback)
        }
        /// C ABI export for this integer specialization.
        ///
        /// # Safety
        /// `this` must be a live handle of this integer width with exclusive access. Query pointers must be null or valid for their lengths; callbacks must be valid and must not reenter the handle or unwind.
        #[no_mangle]
        pub unsafe extern "C" fn $count_united(
            this: *mut c_void,
            query: *const $ty,
            len: usize,
        ) -> $ty {
            count_united::<$ty>(this, query, len)
        }
        /// C ABI export for this integer specialization.
        ///
        /// # Safety
        /// `this` must be a live handle of this integer width with exclusive access. Query pointers must be null or valid for their lengths; callbacks must be valid and must not reenter the handle or unwind.
        #[no_mangle]
        pub unsafe extern "C" fn $update_united(
            this: *mut c_void,
            restrictions: *const $ty,
            len_r: usize,
            substitutuion: *const $ty,
            len_s: usize,
            callback: CUDCallback<$ty>,
        ) -> $ty {
            update_united::<$ty>(this, restrictions, len_r, substitutuion, len_s, callback)
        }
        /// C ABI export for this integer specialization.
        ///
        /// # Safety
        /// `this` must be a live handle of this integer width with exclusive access. Query pointers must be null or valid for their lengths; callbacks must be valid and must not reenter the handle or unwind.
        #[no_mangle]
        pub unsafe extern "C" fn $delete_united(
            this: *mut c_void,
            query: *const $ty,
            len: usize,
            callback: CUDCallback<$ty>,
        ) -> $ty {
            delete_united::<$ty>(this, query, len, callback)
        }
    };
}

export_links!(
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
export_links!(
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
export_links!(
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
export_links!(
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
