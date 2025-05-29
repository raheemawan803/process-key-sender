use anyhow::Result;
use sysinfo::{System, SystemExt, ProcessExt};

#[cfg(windows)]
use winapi::um::winuser::{EnumWindows, GetWindowThreadProcessId, IsWindowVisible, GetWindowTextA};
#[cfg(windows)]
use winapi::shared::windef::HWND;

pub struct ProcessFinder {
    system: System,
}

impl ProcessFinder {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }

    pub fn find_process_window(&mut self, process_name: &str) -> Result<Option<u64>> {
        self.system.refresh_all();

        let process_name_lower = process_name.to_lowercase();

        for (pid, process) in self.system.processes() {
            let name = process.name().to_lowercase();
            if name.contains(&process_name_lower) {
                #[cfg(windows)]
                {
                    if let Some(hwnd) = self.find_window_by_pid_windows(*pid as u32) {
                        return Ok(Some(hwnd as u64));
                    }
                }

                #[cfg(unix)]
                {
                    // For Unix, we'll use the PID as window ID for now
                    return Ok(Some(*pid as u64));
                }
            }
        }

        Ok(None)
    }

    pub fn is_process_running(&mut self, process_name: &str) -> Result<bool> {
        self.system.refresh_all();

        let process_name_lower = process_name.to_lowercase();

        for (_pid, process) in self.system.processes() {
            if process.name().to_lowercase().contains(&process_name_lower) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    #[cfg(windows)]
    fn find_window_by_pid_windows(&self, target_pid: u32) -> Option<HWND> {
        use std::sync::Mutex;

        let result = Mutex::new(None);

        unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: isize) -> i32 {
            let target_pid = lparam as u32;
            let result = &*(lparam as *const Mutex<Option<HWND>>);

            let mut window_pid = 0;
            GetWindowThreadProcessId(hwnd, &mut window_pid);

            if window_pid == target_pid && IsWindowVisible(hwnd) != 0 {
                let mut title = [0u8; 256];
                let len = GetWindowTextA(hwnd, title.as_mut_ptr() as *mut i8, 256);

                if len > 0 {
                    *result.lock().unwrap() = Some(hwnd);
                    return 0; // Stop enumeration
                }
            }

            1 // Continue enumeration
        }

        unsafe {
            EnumWindows(Some(enum_proc), &result as *const _ as isize);
        }

        result.into_inner().unwrap()
    }
}