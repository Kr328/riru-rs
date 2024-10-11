use core::ffi::{c_char, c_int, c_void};

use typed_jni::sys::{jboolean, jclass, jint, jintArray, jlong, jobjectArray, jstring, JNIEnv};

pub const API_VERSION: c_int = 26;

pub type OnModuleLoadedFunc = Option<unsafe extern "C" fn()>;
pub type ShouldSkipUidFunc = Option<unsafe extern "C" fn(uid: c_int) -> c_int>;
pub type NativeForkAndSpecializePreFunc = Option<
    unsafe extern "C" fn(
        env: *mut JNIEnv,
        cls: jclass,
        uid: *mut jint,
        gid: *mut jint,
        gids: *mut jintArray,
        runtime_flags: *mut jint,
        rlimits: *mut jobjectArray,
        mount_external: *mut jint,
        se_info: *mut jstring,
        nice_name: *mut jstring,
        fds_to_close: *mut jintArray,
        fds_to_ignore: *mut jintArray,
        is_child_zygote: *mut jboolean,
        instruction_set: *mut jstring,
        app_data_dir: *mut jstring,
        is_top_app: *mut jboolean,
        pkg_data_info_list: *mut jobjectArray,
        whitelisted_data_info_list: *mut jobjectArray,
        bind_mount_app_data_dirs: *mut jboolean,
        bind_mount_app_storage_dirs: *mut jboolean,
    ),
>;
pub type NativeForkAndSpecializePostFunc = Option<unsafe extern "C" fn(env: *mut JNIEnv, cls: jclass, res: jint)>;
pub type NativeForkSystemServerPreFunc = Option<
    unsafe extern "C" fn(
        env: *mut JNIEnv,
        cls: jclass,
        uid: *mut jint,
        gid: *mut jint,
        gids: *mut jintArray,
        runtime_flags: *mut jint,
        rlimits: *mut jobjectArray,
        permitted_capabilities: *mut jlong,
        effective_capabilities: *mut jlong,
    ),
>;
pub type NativeForkSystemServerPostFunc = Option<unsafe extern "C" fn(env: *mut JNIEnv, cls: jclass, res: jint)>;
pub type NativeSpecializeAppProcessPreFunc = Option<
    unsafe extern "C" fn(
        env: *mut JNIEnv,
        cls: jclass,
        uid: *mut jint,
        gid: *mut jint,
        gids: *mut jintArray,
        runtime_flags: *mut jint,
        rlimits: *mut jobjectArray,
        mount_external: *mut jint,
        se_info: *mut jstring,
        nice_name: *mut jstring,
        start_child_zygote: *mut jboolean,
        instruction_set: *mut jstring,
        app_data_dir: *mut jstring,
        is_top_app: *mut jboolean,
        pkg_data_info_list: *mut jobjectArray,
        whitelisted_data_info_list: *mut jobjectArray,
        bind_mount_app_data_dirs: *mut jboolean,
        bind_mount_app_storage_dirs: *mut jboolean,
    ),
>;
pub type NativeSpecializeAppProcessPostFunc = Option<unsafe extern "C" fn(env: *mut JNIEnv, cls: jclass)>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct RiruModuleInfo {
    pub support_hide: c_int,
    pub version: c_int,
    pub version_name: *const c_char,
    pub on_module_loaded: OnModuleLoadedFunc,
    pub should_skip_uid: ShouldSkipUidFunc,
    pub fork_and_specialize_pre: NativeForkAndSpecializePreFunc,
    pub fork_and_specialize_post: NativeForkAndSpecializePostFunc,
    pub fork_system_server_pre: NativeForkSystemServerPreFunc,
    pub fork_system_server_post: NativeForkSystemServerPostFunc,
    pub specialize_app_process_pre: NativeSpecializeAppProcessPreFunc,
    pub specialize_app_process_post: NativeSpecializeAppProcessPostFunc,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct RiruVersionedModuleInfo {
    pub module_api_version: c_int,
    pub module_info: RiruModuleInfo,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Riru {
    pub riru_api_version: c_int,
    pub unused: *mut c_void,
    pub magisk_module_path: *const c_char,
    pub allow_unload: *mut c_int,
}
