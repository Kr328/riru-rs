#![no_std]

extern crate alloc;

use alloc::{
    boxed::Box,
    ffi::CString,
    string::{String, ToString},
};
use core::{ffi::CStr, ptr::null_mut};

use typed_jni::{define_java_class, Array, Context, JString, LocalObject, TrampolineClass};

use crate::sys::{Riru, RiruModuleInfo, RiruVersionedModuleInfo};

mod sys;

pub struct ForkAndSpecializeArgs<'a> {
    pub uid: Option<&'a mut i32>,
    pub gid: Option<&'a mut i32>,
    pub gids: Option<&'a mut LocalObject<'a, Array<i32>>>,
    pub runtime_flags: Option<&'a mut i32>,
    pub rlimits: Option<&'a mut LocalObject<'a, Array<Array<i32>>>>,
    pub mount_external: Option<&'a mut i32>,
    pub se_info: Option<&'a mut LocalObject<'a, JString>>,
    pub nice_name: Option<&'a mut LocalObject<'a, JString>>,
    pub fds_to_close: Option<&'a mut LocalObject<'a, Array<i32>>>,
    pub fds_to_ignore: Option<&'a mut LocalObject<'a, Array<i32>>>,
    pub is_child_zygote: Option<&'a mut bool>,
    pub instruction_set: Option<&'a mut LocalObject<'a, JString>>,
    pub app_data_dir: Option<&'a mut LocalObject<'a, JString>>,
    pub is_top_app: Option<&'a mut bool>,
    pub pkg_data_info_list: Option<&'a mut LocalObject<'a, Array<JString>>>,
    pub whitelisted_data_info_list: Option<&'a mut LocalObject<'a, Array<JString>>>,
    pub bind_mount_app_data_dirs: Option<&'a mut bool>,
    pub bind_mount_app_storage_dirs: Option<&'a mut bool>,
}

pub struct ForkSystemServerArgs<'a> {
    pub uid: Option<&'a mut i32>,
    pub gid: Option<&'a mut i32>,
    pub gids: Option<&'a mut LocalObject<'a, Array<i32>>>,
    pub runtime_flags: Option<&'a mut i32>,
    pub rlimits: Option<&'a mut LocalObject<'a, Array<Array<i32>>>>,
    pub permitted_capabilities: Option<&'a mut i64>,
    pub effective_capabilities: Option<&'a mut i64>,
}

pub struct SpecializeAppProcessArgs<'a> {
    pub uid: Option<&'a mut i32>,
    pub gid: Option<&'a mut i32>,
    pub gids: Option<&'a mut LocalObject<'a, Array<i32>>>,
    pub runtime_flags: Option<&'a mut i32>,
    pub rlimits: Option<&'a mut LocalObject<'a, Array<Array<i32>>>>,
    pub mount_external: Option<&'a mut i32>,
    pub se_info: Option<&'a mut LocalObject<'a, JString>>,
    pub nice_name: Option<&'a mut LocalObject<'a, JString>>,
    pub start_child_zygote: Option<&'a mut bool>,
    pub instruction_set: Option<&'a mut LocalObject<'a, JString>>,
    pub app_data_dir: Option<&'a mut LocalObject<'a, JString>>,
    pub is_top_app: Option<&'a mut bool>,
    pub pkg_data_info_list: Option<&'a mut LocalObject<'a, Array<JString>>>,
    pub whitelisted_data_info_list: Option<&'a mut LocalObject<'a, Array<JString>>>,
    pub bind_mount_app_data_dirs: Option<&'a mut bool>,
    pub bind_mount_app_storage_dirs: Option<&'a mut bool>,
}

#[derive(Copy, Clone)]
pub enum ForkResult {
    OnParent(i32),
    OnChild,
}

define_java_class!(Zygote, "com.android.internal.os.Zygote");

pub trait Module {
    fn new(api: Api) -> Self;

    fn support_hide(&self) -> bool;
    fn version(&self) -> i32;
    fn version_name(&self) -> String;

    fn should_skip_uid(&mut self, uid: i32) -> bool;
    fn pre_fork_and_specialize<'a>(
        &mut self,
        ctx: &'a Context,
        class: TrampolineClass<'a, Zygote>,
        args: ForkAndSpecializeArgs<'a>,
    );
    fn post_fork_and_specialize<'a>(&mut self, ctx: &'a Context, class: TrampolineClass<'a, Zygote>, result: ForkResult);
    fn pre_fork_system_server<'a>(
        &mut self,
        ctx: &'a Context,
        class: TrampolineClass<'a, Zygote>,
        args: ForkSystemServerArgs<'a>,
    );
    fn post_fork_system_server<'a>(&mut self, ctx: &'a Context, class: TrampolineClass<'a, Zygote>, result: ForkResult);
    fn pre_specialize_app_process<'a>(
        &mut self,
        ctx: &'a Context,
        class: TrampolineClass<'a, Zygote>,
        args: SpecializeAppProcessArgs<'a>,
    );
    fn post_specialize_app_process<'a>(&mut self, ctx: &'a Context, class: TrampolineClass<'a, Zygote>);
}

pub struct Api {
    module_path: String,
    allow_unload: Option<&'static mut i32>,
}

impl Api {
    pub fn set_allow_unload(&mut self, allow_unload: bool) {
        if let Some(mark) = &mut self.allow_unload {
            **mark = if allow_unload { 1 } else { 0 }
        }
    }

    pub fn get_module_path(&self) -> &str {
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

unsafe extern "C" fn module_should_skip_uid<M: Module>(uid: i32) -> i32 {
    if (*MODULE.cast::<M>()).should_skip_uid(uid) {
        1
    } else {
        0
    }
}

unsafe extern "C" fn module_fork_and_specialize_pre<'a, M: Module>(
    ctx: &'a Context,
    clazz: TrampolineClass<'a, Zygote>,
    uid: Option<&'a mut i32>,
    gid: Option<&'a mut i32>,
    gids: Option<&'a mut LocalObject<'a, Array<i32>>>,
    runtime_flags: Option<&'a mut i32>,
    rlimits: Option<&'a mut LocalObject<'a, Array<Array<i32>>>>,
    mount_external: Option<&'a mut i32>,
    se_info: Option<&'a mut LocalObject<'a, JString>>,
    nice_name: Option<&'a mut LocalObject<'a, JString>>,
    fds_to_close: Option<&'a mut LocalObject<'a, Array<i32>>>,
    fds_to_ignore: Option<&'a mut LocalObject<'a, Array<i32>>>,
    is_child_zygote: Option<&'a mut bool>,
    instruction_set: Option<&'a mut LocalObject<'a, JString>>,
    app_data_dir: Option<&'a mut LocalObject<'a, JString>>,
    is_top_app: Option<&'a mut bool>,
    pkg_data_info_list: Option<&'a mut LocalObject<'a, Array<JString>>>,
    whitelisted_data_info_list: Option<&'a mut LocalObject<'a, Array<JString>>>,
    bind_mount_app_data_dirs: Option<&'a mut bool>,
    bind_mount_app_storage_dirs: Option<&'a mut bool>,
) {
    let args = ForkAndSpecializeArgs {
        uid,
        gid,
        gids,
        runtime_flags,
        rlimits,
        mount_external,
        se_info,
        nice_name,
        fds_to_close,
        fds_to_ignore,
        is_child_zygote,
        instruction_set,
        app_data_dir,
        is_top_app,
        pkg_data_info_list,
        whitelisted_data_info_list,
        bind_mount_app_data_dirs,
        bind_mount_app_storage_dirs,
    };

    (*MODULE.cast::<M>()).pre_fork_and_specialize(ctx, clazz, args);
}

unsafe extern "C" fn module_fork_and_specialize_post<'a, M: Module>(
    ctx: &'a Context,
    clazz: TrampolineClass<'a, Zygote>,
    res: i32,
) {
    let result = if res == 0 {
        ForkResult::OnChild
    } else {
        ForkResult::OnParent(res)
    };

    (*MODULE.cast::<M>()).post_fork_and_specialize(ctx, clazz, result);
}

unsafe extern "C" fn module_fork_system_server_pre<'a, M: Module>(
    ctx: &'a Context,
    clazz: TrampolineClass<'a, Zygote>,
    uid: Option<&'a mut i32>,
    gid: Option<&'a mut i32>,
    gids: Option<&'a mut LocalObject<'a, Array<i32>>>,
    runtime_flags: Option<&'a mut i32>,
    rlimits: Option<&'a mut LocalObject<'a, Array<Array<i32>>>>,
    permitted_capabilities: Option<&'a mut i64>,
    effective_capabilities: Option<&'a mut i64>,
) {
    let args = ForkSystemServerArgs {
        uid,
        gid,
        gids,
        runtime_flags,
        rlimits,
        permitted_capabilities,
        effective_capabilities,
    };

    (*MODULE.cast::<M>()).pre_fork_system_server(ctx, clazz, args);
}

unsafe extern "C" fn module_fork_system_server_post<'a, M: Module>(
    ctx: &'a Context,
    clazz: TrampolineClass<'a, Zygote>,
    res: i32,
) {
    let result = if res == 0 {
        ForkResult::OnChild
    } else {
        ForkResult::OnParent(res)
    };

    (*MODULE.cast::<M>()).post_fork_system_server(ctx, clazz, result);
}

unsafe extern "C" fn module_specialize_app_process_pre<'a, M: Module>(
    ctx: &'a Context,
    clazz: TrampolineClass<'a, Zygote>,
    uid: Option<&'a mut i32>,
    gid: Option<&'a mut i32>,
    gids: Option<&'a mut LocalObject<'a, Array<i32>>>,
    runtime_flags: Option<&'a mut i32>,
    rlimits: Option<&'a mut LocalObject<'a, Array<Array<i32>>>>,
    mount_external: Option<&'a mut i32>,
    se_info: Option<&'a mut LocalObject<'a, JString>>,
    nice_name: Option<&'a mut LocalObject<'a, JString>>,
    start_child_zygote: Option<&'a mut bool>,
    instruction_set: Option<&'a mut LocalObject<'a, JString>>,
    app_data_dir: Option<&'a mut LocalObject<'a, JString>>,
    is_top_app: Option<&'a mut bool>,
    pkg_data_info_list: Option<&'a mut LocalObject<'a, Array<JString>>>,
    whitelisted_data_info_list: Option<&'a mut LocalObject<'a, Array<JString>>>,
    bind_mount_app_data_dirs: Option<&'a mut bool>,
    bind_mount_app_storage_dirs: Option<&'a mut bool>,
) {
    let args = SpecializeAppProcessArgs {
        uid,
        gid,
        gids,
        runtime_flags,
        rlimits,
        mount_external,
        se_info,
        nice_name,
        start_child_zygote,
        instruction_set,
        app_data_dir,
        is_top_app,
        pkg_data_info_list,
        whitelisted_data_info_list,
        bind_mount_app_data_dirs,
        bind_mount_app_storage_dirs,
    };

    (*MODULE.cast::<M>()).pre_specialize_app_process(ctx, clazz, args);
}

unsafe extern "C" fn module_specialize_app_process_post<'a, M: Module>(ctx: &'a Context, clazz: TrampolineClass<'a, Zygote>) {
    (*MODULE.cast::<M>()).post_specialize_app_process(ctx, clazz);
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
        module_path: CStr::from_ptr((*riru).magisk_module_path).to_str().unwrap().to_string(),
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
            fork_and_specialize_pre: Some(core::mem::transmute(module_fork_and_specialize_pre::<M> as *const ())),
            fork_and_specialize_post: Some(core::mem::transmute(module_fork_and_specialize_post::<M> as *const ())),
            fork_system_server_pre: Some(core::mem::transmute(module_fork_system_server_pre::<M> as *const ())),
            fork_system_server_post: Some(core::mem::transmute(module_fork_system_server_post::<M> as *const ())),
            specialize_app_process_pre: Some(core::mem::transmute(module_specialize_app_process_pre::<M> as *const ())),
            specialize_app_process_post: Some(core::mem::transmute(module_specialize_app_process_post::<M> as *const ())),
        },
    };

    MODULE = Box::leak(Box::new(module)) as *mut M as *mut ();

    Box::into_raw(Box::new(info))
}
