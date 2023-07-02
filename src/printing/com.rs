use log::error;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

pub fn init() -> Result<(), ()> {
    let result = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    if let Err(e) = result {
        error!("couldn't initialise com interface: {}", e.message());
        return Err(());
    }

    return Ok(());
}
