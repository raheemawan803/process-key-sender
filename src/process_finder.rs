use anyhow::Result;
use sysinfo::{System};

pub struct ProcessFinder {
    system: System,
}

impl Clone for ProcessFinder {
    fn clone(&self) -> Self {
        Self {
            system: System::new(),
        }
    }
}

impl ProcessFinder {
    pub fn new() -> Self {
        Self {
            system: System::new(),
        }
    }

    pub fn find_process_window(&mut self, process_name: &str) -> Result<Option<u64>> {
        // Refresh all processes
        self.system.refresh_processes();

        let process_name_lower = process_name.to_lowercase();

        for (pid, process) in self.system.processes() {
            let name = process.name().to_lowercase();
            if name.contains(&process_name_lower) {
                #[cfg(windows)]
                {
                    // For Windows, we'll use a simpler approach - just return the PID as window ID
                    // The key sender can work with this approach
                    return Ok(Some(pid.as_u32() as u64));
                }

                #[cfg(unix)]
                {
                    // For Unix, we'll use the PID as window ID for now
                    return Ok(Some(pid.as_u32() as u64));
                }
            }
        }

        Ok(None)
    }
    
    #[deprecated]
    #[allow(dead_code)]
    pub fn is_process_running(&mut self, process_name: &str) -> Result<bool> {
        // Refresh all processes
        self.system.refresh_processes();

        let process_name_lower = process_name.to_lowercase();

        for (_pid, process) in self.system.processes() {
            let name = process.name().to_lowercase();
            if name.contains(&process_name_lower) {
                return Ok(true);
            }
        }

        Ok(false)
    }
}