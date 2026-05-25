use std::path::PathBuf;

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux", test))]
fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
pub fn get_config_dir(qualifier: &str, org: &str, app: &str) -> Option<PathBuf> {
    directories::ProjectDirs::from(&sanitize(qualifier), &sanitize(org), &sanitize(app))
        .map(|proj_dirs| proj_dirs.config_dir().to_path_buf())
}

#[cfg(target_os = "android")]
pub fn get_config_dir(_q: &str, _o: &str, _a: &str) -> Option<PathBuf> {
    let ctx = ndk_context::android_context();
    // SAFETY: android_context is initialized by android-activity before Rust runs.
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) };
    vm.attach_current_thread(|env| -> jni::errors::Result<PathBuf> {
        // SAFETY: ctx.context() is the Android Context jobject, valid for the app lifetime.
        let activity = unsafe { jni::objects::JObject::from_raw(env, ctx.context().cast()) };
        let files_dir = env
            .call_method(
                &activity,
                jni::jni_str!("getFilesDir"),
                jni::jni_sig!("()Ljava/io/File;"),
                &[],
            )?
            .l()?;
        let path_obj = env
            .call_method(
                &files_dir,
                jni::jni_str!("getAbsolutePath"),
                jni::jni_sig!("()Ljava/lang/String;"),
                &[],
            )?
            .l()?;
        let path_jstr = env.cast_local::<jni::objects::JString>(path_obj)?;
        let path = path_jstr.try_to_string(env)?;
        Ok(PathBuf::from(path).join("config"))
    })
    .ok()
}

#[cfg(target_os = "ios")]
pub fn get_config_dir(_q: &str, _o: &str, _a: &str) -> Option<PathBuf> {
    use objc::{class, msg_send, sel, sel_impl};
    use std::ffi::CStr;

    unsafe {
        let file_manager: *mut objc::runtime::Object =
            msg_send![class!(NSFileManager), defaultManager];
        // 14 = NSApplicationSupportDirectory, 1 = NSUserDomainMask
        let urls: *mut objc::runtime::Object =
            msg_send![file_manager, URLsForDirectory:14 inDomains:1];
        let count: usize = msg_send![urls, count];

        if count == 0 {
            return None;
        }

        let url: *mut objc::runtime::Object = msg_send![urls, firstObject];
        let path_nsstring: *mut objc::runtime::Object = msg_send![url, path];
        let path_ptr: *const std::ffi::c_char = msg_send![path_nsstring, UTF8String];

        if path_ptr.is_null() {
            return None;
        }
        Some(PathBuf::from(
            CStr::from_ptr(path_ptr).to_string_lossy().into_owned(),
        ))
    }
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "android",
    target_os = "ios"
)))]
pub fn get_config_dir(_q: &str, _o: &str, _a: &str) -> Option<PathBuf> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize() {
        assert_eq!(sanitize("My App"), "My_App");
        assert_eq!(sanitize("Acme-Corp!"), "Acme-Corp_");
        assert_eq!(sanitize(".. /path"), "____path");
        assert_eq!(sanitize("Valid_Name-123"), "Valid_Name-123");
    }
}
