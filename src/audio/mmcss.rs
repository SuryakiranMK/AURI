// AURI Windows MMCSS (Multimedia Class Scheduler Service) Registration
// Grants real-time scheduler priority to WASAPI audio threads

#[cfg(windows)]
pub struct MmcssRegistration {
    handle: *mut std::ffi::c_void,
}

#[cfg(windows)]
unsafe impl Send for MmcssRegistration {}
#[cfg(windows)]
unsafe impl Sync for MmcssRegistration {}

#[cfg(windows)]
impl MmcssRegistration {
    pub fn register_audio_thread() -> Option<Self> {
        #[link(name = "avrt")]
        extern "system" {
            fn AvSetMmThreadCharacteristicsW(
                task_name: *const u16,
                task_index: *mut u32,
            ) -> *mut std::ffi::c_void;
        }

        let task_name: Vec<u16> = "Audio\0".encode_utf16().collect();
        let mut task_index = 0u32;
        let handle = unsafe { AvSetMmThreadCharacteristicsW(task_name.as_ptr(), &mut task_index) };

        if !handle.is_null() {
            Some(Self { handle })
        } else {
            None
        }
    }
}

#[cfg(windows)]
impl Drop for MmcssRegistration {
    fn drop(&mut self) {
        #[link(name = "avrt")]
        extern "system" {
            fn AvRevertMmThreadCharacteristics(handle: *mut std::ffi::c_void) -> i32;
        }

        if !self.handle.is_null() {
            unsafe {
                AvRevertMmThreadCharacteristics(self.handle);
            }
        }
    }
}

#[cfg(not(windows))]
pub struct MmcssRegistration;

#[cfg(not(windows))]
impl MmcssRegistration {
    pub fn register_audio_thread() -> Option<Self> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mmcss_init_and_drop() {
        let _reg = MmcssRegistration::register_audio_thread();
        // Should either succeed or return None without panic
    }
}

