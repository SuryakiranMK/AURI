// Project AURI: Working Set Memory Trimmer
// Windows native working set purging via EmptyWorkingSet()
// Drastically drops AURI memory footprint down to < 25-35 MB when entering Gaming Mode.

use std::ffi::c_void;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct ProcessMemoryCounters {
    cb: u32,
    page_fault_count: u32,
    peak_working_set_size: usize,
    working_set_size: usize,
    quota_peak_paged_pool_usage: usize,
    quota_paged_pool_usage: usize,
    quota_peak_non_paged_pool_usage: usize,
    quota_non_paged_pool_usage: usize,
    pagefile_usage: usize,
    peak_pagefile_usage: usize,
}

#[link(name = "psapi")]
extern "system" {
    fn EmptyWorkingSet(hProcess: *mut c_void) -> i32;
    fn GetProcessMemoryInfo(
        hProcess: *mut c_void,
        ppsmemCounters: *mut ProcessMemoryCounters,
        cb: u32,
    ) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> *mut c_void;
}

/// Statistics report of a working set trim operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkingSetReport {
    pub before_bytes: usize,
    pub after_bytes: usize,
    pub freed_bytes: usize,
}

impl WorkingSetReport {
    /// Returns the freed memory in megabytes.
    pub fn freed_mb(&self) -> f64 {
        self.freed_bytes as f64 / (1024.0 * 1024.0)
    }

    /// Returns the remaining working set in megabytes.
    pub fn after_mb(&self) -> f64 {
        self.after_bytes as f64 / (1024.0 * 1024.0)
    }
}

/// Retrieves the current process Working Set (RAM in physical memory) in bytes.
pub fn get_working_set_size_bytes() -> Option<usize> {
    unsafe {
        let process = GetCurrentProcess();
        let mut counters = ProcessMemoryCounters::default();
        counters.cb = std::mem::size_of::<ProcessMemoryCounters>() as u32;

        if GetProcessMemoryInfo(
            process,
            &mut counters,
            counters.cb,
        ) != 0
        {
            Some(counters.working_set_size)
        } else {
            None
        }
    }
}

/// Trims the process working set by discarding unneeded pages from RAM.
/// Highly effective when transitioning into Gaming Mode or hiding the GUI.
pub fn trim_working_set() -> Result<WorkingSetReport, &'static str> {
    let before = get_working_set_size_bytes().unwrap_or(0);

    unsafe {
        let process = GetCurrentProcess();
        if EmptyWorkingSet(process) == 0 {
            return Err("Failed to call EmptyWorkingSet on current process");
        }
    }

    let after = get_working_set_size_bytes().unwrap_or(before);
    let freed_bytes = before.saturating_sub(after);

    Ok(WorkingSetReport {
        before_bytes: before,
        after_bytes: after,
        freed_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_working_set_query() {
        let ws = get_working_set_size_bytes();
        assert!(ws.is_some(), "Working set size should be queryable");
        assert!(ws.unwrap() > 0, "Working set size should be non-zero");
    }

    #[test]
    fn test_trim_working_set() {
        let result = trim_working_set();
        assert!(result.is_ok(), "EmptyWorkingSet should succeed");
        let report = result.unwrap();
        assert!(report.before_bytes > 0);
        assert!(report.after_bytes > 0);
        // Working set after purge should typically be lower or equal
        assert!(report.after_bytes <= report.before_bytes);
    }
}

