// Windows Multimedia Class Scheduler Service (MMCSS) Integration
// Grants the real-time audio thread elevated priority so games never cause stutter.

pub struct MmcssRegistration {
    #[cfg(windows)]
    handle: *mut std::ffi::c_void,
}

#[cfg(windows)]
#[link(name = "avrt")]
extern "system" {
    fn AvSetMmThreadCharacteristicsW(task_name: *const u16, task_index: *mut u32) -> *mut std::ffi::c_void;
    fn AvRevertMmThreadCharacteristics(avrt_handle: *mut std::ffi::c_void) -> i32;
}

impl MmcssRegistration {
    pub fn register_audio() -> Self {
        #[cfg(windows)]
        {
            let mut task_index = 0u32;
            // UTF-16 "Audio\0"
            let task_name: [u16; 6] = [0x0041, 0x0075, 0x0064, 0x0069, 0x006F, 0x0000];
            unsafe {
                let handle = AvSetMmThreadCharacteristicsW(task_name.as_ptr(), &mut task_index);
                if !handle.is_null() {
                    println!("[AURI MMCSS] Registered audio thread with real-time MMCSS priority (task index: {})", task_index);
                    return MmcssRegistration { handle };
                } else {
                    println!("[AURI MMCSS] Note: MMCSS registration returned null; running with standard OS scheduling");
                }
            }
        }
        MmcssRegistration {
            #[cfg(windows)]
            handle: std::ptr::null_mut(),
        }
    }
}

impl Drop for MmcssRegistration {
    fn drop(&mut self) {
        #[cfg(windows)]
        {
            if !self.handle.is_null() {
                unsafe {
                    AvRevertMmThreadCharacteristics(self.handle);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mmcss_registration_and_drop() {
        let registration = MmcssRegistration::register_audio();
        // Dropping the registration should revert cleanly without panic
        drop(registration);
    }
}

unsafe impl Send for MmcssRegistration {}
unsafe impl Sync for MmcssRegistration {}
