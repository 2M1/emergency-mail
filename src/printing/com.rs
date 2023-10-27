#[cfg(feature = "xps")]
use log::error;
#[cfg(feature = "xps")]
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

#[cfg(feature = "xps")]
pub fn init() -> Result<(), ()> {
    let result = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    if let Err(e) = result {
        error!("couldn't initialise com interface: {}", e.message());
        return Err(());
    }

    return Ok(());
}

#[cfg(not(feature = "xps"))]
pub fn init() -> Result<(), ()> {
    // empty init to keep main function untouched when not compiling for windows
    Ok(())
}
