use std::{
    ffi::{CStr, CString},
    path::{Path, PathBuf},
    ptr::null_mut,
};

use jni_sys::{jboolean, jclass, jint, jintArray, jlong, jobjectArray, jstring, JNIEnv};
use libc::{c_int, gid_t, uid_t};

use crate::sys::{Riru, RiruModuleInfo, RiruVersionedModuleInfo};

mod sys;

pub struct ForkAndSpecializeArgs<'a> {
    pub uid: Option<&'a mut jint>,
    pub gid: Option<&'a mut jint>,
    pub gids: Option<&'a mut jintArray>,
    pub runtime_flags: Option<&'a mut jint>,
    pub rlimits: Option<&'a mut jobjectArray>,
    pub mount_external: Option<&'a mut jint>,
    pub se_info: Option<&'a mut jstring>,
    pub nice_name: Option<&'a mut jstring>,
    pub fds_to_close: Option<&'a mut jintArray>,
    pub fds_to_ignore: Option<&'a mut jintArray>,
    pub is_child_zygote: Option<&'a mut jboolean>,
    pub instruction_set: Option<&'a mut jstring>,
    pub app_data_dir: Option<&'a mut jstring>,
    pub is_top_app: Option<&'a mut jboolean>,
    pub pkg_data_info_list: Option<&'a mut jobjectArray>,
    pub whitelisted_data_info_list: Option<&'a mut jobjectArray>,
    pub bind_mount_app_data_dirs: Option<&'a mut jboolean>,
    pub bind_mount_app_storage_dirs: Option<&'a mut jboolean>,
}

pub struct ForkSystemServerArgs<'a> {
    pub uid: Option<&'a mut uid_t>,
    pub gid: Option<&'a mut gid_t>,
    pub gids: Option<&'a mut jintArray>,
    pub runtime_flags: Option<&'a mut jint>,
    pub rlimits: Option<&'a mut jobjectArray>,
    pub permitted_capabilities: Option<&'a mut jlong>,
    pub effective_capabilities: Option<&'a mut jlong>,
}

pub struct SpecializeAppProcessArgs<'a> {
    pub uid: Option<&'a mut jint>,
    pub gid: Option<&'a mut jint>,
    pub gids: Option<&'a mut jintArray>,
    pub runtime_flags: Option<&'a mut jint>,
    pub rlimits: Option<&'a mut jobjectArray>,
    pub mount_external: Option<&'a mut jint>,
    pub se_info: Option<&'a mut jstring>,
    pub nice_name: Option<&'a mut jstring>,
    pub start_child_zygote: Option<&'a mut jboolean>,
    pub instruction_set: Option<&'a mut jstring>,
    pub app_data_dir: Option<&'a mut jstring>,
    pub is_top_app: Option<&'a mut jboolean>,
    pub pkg_data_info_list: Option<&'a mut jobjectArray>,
    pub whitelisted_data_info_list: Option<&'a mut jobjectArray>,
    pub bind_mount_app_data_dirs: Option<&'a mut jboolean>,
    pub bind_mount_app_storage_dirs: Option<&'a mut jboolean>,
}

pub enum ForkResult {
    OnParent(libc::pid_t),
    OnChild,
}

pub trait Module {
    fn new(api: Api) -> Self;

    fn support_hide(&self) -> bool;
    fn version(&self) -> i32;
    fn version_name(&self) -> String;

    fn should_skip_uid(&mut self, uid: jint) -> bool;
    fn pre_fork_and_specialize(&mut self, env: *mut JNIEnv, args: &mut ForkAndSpecializeArgs);
    fn post_fork_and_specialize(&mut self, env: *mut JNIEnv, result: ForkResult);
    fn pre_fork_system_server(&mut self, env: *mut JNIEnv, args: &mut ForkSystemServerArgs);
    fn post_fork_system_server(&mut self, env: *mut JNIEnv, result: ForkResult);
    fn pre_specialize_app_process(&mut self, env: *mut JNIEnv, args: &mut SpecializeAppProcessArgs);
    fn post_specialize_app_process(&mut self, env: *mut JNIEnv);
}

pub struct Api {
    module_path: PathBuf,
    allow_unload: Option<&'static mut c_int>,
}

impl Api {
    pub fn set_allow_unload(&mut self, allow_unload: bool) {
        if let Some(mark) = &mut self.allow_unload {
            **mark = if allow_unload { 1 } else { 0 }
        }
    }

    pub fn get_module_path(&self) -> &Path {
        &self.module_path
    }
}

#[macro_export]
macro_rules! register_riru_module {
    ($module:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn init(riru: *const ::std::ffi::c_void) -> *mut ::std::ffi::c_void {
            $crate::_module_entry::<$module>(riru.cast()).cast()
        }
    };
}

static mut MODULE: *mut () = null_mut();

unsafe extern "C" fn module_should_skip_uid<M: Module>(uid: c_int) -> c_int {
    if (*MODULE.cast::<M>()).should_skip_uid(uid) {
        1
    } else {
        0
    }
}

unsafe extern "C" fn module_fork_and_specialize_pre<M: Module>(
    env: *mut JNIEnv,
    _: jclass,
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
) {
    let mut args = ForkAndSpecializeArgs {
        uid: uid.as_mut(),
        gid: gid.as_mut(),
        gids: gids.as_mut(),
        runtime_flags: runtime_flags.as_mut(),
        rlimits: rlimits.as_mut(),
        mount_external: mount_external.as_mut(),
        se_info: se_info.as_mut(),
        nice_name: nice_name.as_mut(),
        fds_to_close: fds_to_close.as_mut(),
        fds_to_ignore: fds_to_ignore.as_mut(),
        is_child_zygote: is_child_zygote.as_mut(),
        instruction_set: instruction_set.as_mut(),
        app_data_dir: app_data_dir.as_mut(),
        is_top_app: is_top_app.as_mut(),
        pkg_data_info_list: pkg_data_info_list.as_mut(),
        whitelisted_data_info_list: whitelisted_data_info_list.as_mut(),
        bind_mount_app_data_dirs: bind_mount_app_data_dirs.as_mut(),
        bind_mount_app_storage_dirs: bind_mount_app_storage_dirs.as_mut(),
    };

    (*MODULE.cast::<M>()).pre_fork_and_specialize(env, &mut args);
}

unsafe extern "C" fn module_fork_and_specialize_post<M: Module>(env: *mut JNIEnv, _: jclass, res: jint) {
    let result = if res == 0 {
        ForkResult::OnChild
    } else {
        ForkResult::OnParent(res as libc::pid_t)
    };

    (*MODULE.cast::<M>()).post_fork_and_specialize(env, result);
}

unsafe extern "C" fn module_fork_system_server_pre<M: Module>(
    env: *mut JNIEnv,
    _: jclass,
    uid: *mut uid_t,
    gid: *mut gid_t,
    gids: *mut jintArray,
    runtime_flags: *mut jint,
    rlimits: *mut jobjectArray,
    permitted_capabilities: *mut jlong,
    effective_capabilities: *mut jlong,
) {
    let mut args = ForkSystemServerArgs {
        uid: uid.as_mut(),
        gid: gid.as_mut(),
        gids: gids.as_mut(),
        runtime_flags: runtime_flags.as_mut(),
        rlimits: rlimits.as_mut(),
        permitted_capabilities: permitted_capabilities.as_mut(),
        effective_capabilities: effective_capabilities.as_mut(),
    };

    (*MODULE.cast::<M>()).pre_fork_system_server(env, &mut args);
}

unsafe extern "C" fn module_fork_system_server_post<M: Module>(env: *mut JNIEnv, _: jclass, res: jint) {
    let result = if res == 0 {
        ForkResult::OnChild
    } else {
        ForkResult::OnParent(res as libc::pid_t)
    };

    (*MODULE.cast::<M>()).post_fork_system_server(env, result);
}

unsafe extern "C" fn module_specialize_app_process_pre<M: Module>(
    env: *mut JNIEnv,
    _: jclass,
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
) {
    let mut args = SpecializeAppProcessArgs {
        uid: uid.as_mut(),
        gid: gid.as_mut(),
        gids: gids.as_mut(),
        runtime_flags: runtime_flags.as_mut(),
        rlimits: rlimits.as_mut(),
        mount_external: mount_external.as_mut(),
        se_info: se_info.as_mut(),
        nice_name: nice_name.as_mut(),
        start_child_zygote: start_child_zygote.as_mut(),
        instruction_set: instruction_set.as_mut(),
        app_data_dir: app_data_dir.as_mut(),
        is_top_app: is_top_app.as_mut(),
        pkg_data_info_list: pkg_data_info_list.as_mut(),
        whitelisted_data_info_list: whitelisted_data_info_list.as_mut(),
        bind_mount_app_data_dirs: bind_mount_app_data_dirs.as_mut(),
        bind_mount_app_storage_dirs: bind_mount_app_storage_dirs.as_mut(),
    };

    (*MODULE.cast::<M>()).pre_specialize_app_process(env, &mut args);
}

unsafe extern "C" fn module_specialize_app_process_post<M: Module>(env: *mut JNIEnv, _: jclass) {
    (*MODULE.cast::<M>()).post_specialize_app_process(env);
}

#[doc(hidden)]
pub unsafe fn _module_entry<M: Module>(riru: *const Riru) -> *mut RiruVersionedModuleInfo {
    if !MODULE.is_null() {
        panic!("already registered");
    }

    if (*riru).riru_api_version < sys::API_VERSION {
        return null_mut();
    }

    let api = Api {
        module_path: PathBuf::from(CStr::from_ptr((*riru).magisk_module_path).to_str().unwrap().to_string()),
        allow_unload: (*riru).allow_unload.as_mut(),
    };

    let module = M::new(api);

    let info = RiruVersionedModuleInfo {
        module_api_version: sys::API_VERSION,
        module_info: RiruModuleInfo {
            support_hide: if module.support_hide() { 1 } else { 0 },
            version: module.version(),
            version_name: CString::new(module.version_name()).unwrap().into_raw(),
            on_module_loaded: None,
            should_skip_uid: Some(module_should_skip_uid::<M>),
            fork_and_specialize_pre: Some(module_fork_and_specialize_pre::<M>),
            fork_and_specialize_post: Some(module_fork_and_specialize_post::<M>),
            fork_system_server_pre: Some(module_fork_system_server_pre::<M>),
            fork_system_server_post: Some(module_fork_system_server_post::<M>),
            specialize_app_process_pre: Some(module_specialize_app_process_pre::<M>),
            specialize_app_process_post: Some(module_specialize_app_process_post::<M>),
        },
    };

    MODULE = Box::leak(Box::new(module)) as *mut M as *mut ();

    Box::into_raw(Box::new(info))
}
