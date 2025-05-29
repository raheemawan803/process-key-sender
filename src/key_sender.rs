use anyhow::Result;
use std::collections::HashMap;

#[cfg(windows)]
use winapi::um::winuser::{PostMessageA, WM_KEYDOWN, WM_KEYUP, VK_SPACE, VK_RETURN, VK_TAB, VK_ESCAPE, VK_SHIFT, VK_CONTROL, VK_MENU};

pub struct KeySender {
    #[cfg(windows)]
    key_map: HashMap<String, u32>,
}

impl KeySender {
    pub fn new() -> Result<Self> {
        #[cfg(windows)]
        {
            let mut key_map = HashMap::new();

            // Special keys
            key_map.insert("space".to_string(), VK_SPACE as u32);
            key_map.insert("enter".to_string(), VK_RETURN as u32);
            key_map.insert("return".to_string(), VK_RETURN as u32);
            key_map.insert("tab".to_string(), VK_TAB as u32);
            key_map.insert("escape".to_string(), VK_ESCAPE as u32);
            key_map.insert("esc".to_string(), VK_ESCAPE as u32);
            key_map.insert("shift".to_string(), VK_SHIFT as u32);
            key_map.insert("ctrl".to_string(), VK_CONTROL as u32);
            key_map.insert("control".to_string(), VK_CONTROL as u32);
            key_map.insert("alt".to_string(), VK_MENU as u32);

            // Function keys
            for i in 1..=12 {
                key_map.insert(format!("f{}", i), (0x70 + i - 1) as u32); // VK_F1 to VK_F12
            }

            // Number keys
            for i in 0..=9 {
                key_map.insert(i.to_string(), (0x30 + i) as u32); // VK_0 to VK_9
            }

            // Arrow keys
            key_map.insert("left".to_string(), 0x25); // VK_LEFT
            key_map.insert("up".to_string(), 0x26); // VK_UP
            key_map.insert("right".to_string(), 0x27); // VK_RIGHT
            key_map.insert("down".to_string(), 0x28); // VK_DOWN

            // Additional common keys
            key_map.insert("backspace".to_string(), 0x08); // VK_BACK
            key_map.insert("delete".to_string(), 0x2E); // VK_DELETE
            key_map.insert("home".to_string(), 0x24); // VK_HOME
            key_map.insert("end".to_string(), 0x23); // VK_END
            key_map.insert("pageup".to_string(), 0x21); // VK_PRIOR
            key_map.insert("pagedown".to_string(), 0x22); // VK_NEXT

            Ok(Self { key_map })
        }

        #[cfg(unix)]
        {
            Ok(Self)
        }
    }

    pub fn send_key_to_window(&self, window_id: u64, key: &str) -> Result<()> {
        #[cfg(windows)]
        {
            self.send_key_windows(window_id as isize, key)
        }

        #[cfg(unix)]
        {
            self.send_key_unix(window_id, key)
        }
    }

    #[cfg(windows)]
    fn send_key_windows(&self, hwnd: isize, key: &str) -> Result<()> {
        // Handle key combinations (e.g., "ctrl+c")
        if key.contains('+') {
            return self.send_key_combination_windows(hwnd, key);
        }

        let vk_code = self.parse_key_windows(key)?;

        unsafe {
            PostMessageA(hwnd as *mut _, WM_KEYDOWN, vk_code as usize, 0);
            std::thread::sleep(std::time::Duration::from_millis(50));
            PostMessageA(hwnd as *mut _, WM_KEYUP, vk_code as usize, 0);
        }

        Ok(())
    }

    #[cfg(windows)]
    fn send_key_combination_windows(&self, hwnd: isize, key_combo: &str) -> Result<()> {
        let parts: Vec<&str> = key_combo.split('+').map(|s| s.trim()).collect();
        if parts.len() < 2 {
            anyhow::bail!("Invalid key combination format: {}", key_combo);
        }

        let mut modifier_codes = Vec::new();
        let main_key = parts.last().unwrap();

        // Parse modifiers
        for modifier in &parts[..parts.len() - 1] {
            let vk_code = self.parse_key_windows(modifier)?;
            modifier_codes.push(vk_code);
        }

        // Parse main key
        let main_key_code = self.parse_key_windows(main_key)?;

        unsafe {
            // Press modifiers
            for &modifier_code in &modifier_codes {
                PostMessageA(hwnd as *mut _, WM_KEYDOWN, modifier_code as usize, 0);
            }

            // Press main key
            PostMessageA(hwnd as *mut _, WM_KEYDOWN, main_key_code as usize, 0);
            std::thread::sleep(std::time::Duration::from_millis(50));
            PostMessageA(hwnd as *mut _, WM_KEYUP, main_key_code as usize, 0);

            // Release modifiers (in reverse order)
            for &modifier_code in modifier_codes.iter().rev() {
                PostMessageA(hwnd as *mut _, WM_KEYUP, modifier_code as usize, 0);
            }
        }

        Ok(())
    }

    #[cfg(windows)]
    fn parse_key_windows(&self, key: &str) -> Result<u32> {
        let key_lower = key.to_lowercase();

        // Check special keys first
        if let Some(&vk_code) = self.key_map.get(&key_lower) {
            return Ok(vk_code);
        }

        // Handle single letter keys
        if key_lower.len() == 1 {
            let ch = key_lower.chars().next().unwrap();
            if ch.is_ascii_alphabetic() {
                return Ok(ch.to_uppercase().next().unwrap() as u32);
            }
        }

        anyhow::bail!("Unsupported key: {}", key)
    }

    #[cfg(unix)]
    fn send_key_unix(&self, _window_id: u64, _key: &str) -> Result<()> {
        // TODO: Implement X11 key sending
        anyhow::bail!("Unix key sending not yet implemented")
    }
}