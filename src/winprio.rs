use windows::Win32::System::Threading::SetPriorityClass;
use windows::Win32::System::Threading::GetCurrentProcess;


#[cfg(target_os = "windows")]
pub fn set_process_priority() {
    let res = unsafe {
        SetPriorityClass(GetCurrentProcess(), windows::Win32::System::Threading::REALTIME_PRIORITY_CLASS)
    };

    if res.as_bool() {
        log::info!("Set process priority to REALTIME_PRIORITY_CLASS");
    } else {
        log::warn!("Couldn't set process priority to REALTIME_PRIORITY_CLASS");
    }
}

#[cfg(not(target_os = "windows"))]
pub fn set_priority() {
    log::trace!("Setting process priority is not implemented on this platform");
}