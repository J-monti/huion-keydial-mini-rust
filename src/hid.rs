use evdev::Key;

/// Convert USB HID usage ID to evdev Key
pub fn hid_to_key(usage: u8) -> Option<Key> {
    match usage {
        0x04 => Some(Key::KEY_A),  0x05 => Some(Key::KEY_B),
        0x06 => Some(Key::KEY_C),  0x07 => Some(Key::KEY_D),
        0x08 => Some(Key::KEY_E),  0x09 => Some(Key::KEY_F),
        0x0A => Some(Key::KEY_G),  0x0B => Some(Key::KEY_H),
        0x0C => Some(Key::KEY_I),  0x0D => Some(Key::KEY_J),
        0x0E => Some(Key::KEY_K),  0x0F => Some(Key::KEY_L),
        0x10 => Some(Key::KEY_M),  0x11 => Some(Key::KEY_N),
        0x12 => Some(Key::KEY_O),  0x13 => Some(Key::KEY_P),
        0x14 => Some(Key::KEY_Q),  0x15 => Some(Key::KEY_R),
        0x16 => Some(Key::KEY_S),  0x17 => Some(Key::KEY_T),
        0x18 => Some(Key::KEY_U),  0x19 => Some(Key::KEY_V),
        0x1A => Some(Key::KEY_W),  0x1B => Some(Key::KEY_X),
        0x1C => Some(Key::KEY_Y),  0x1D => Some(Key::KEY_Z),
        0x1E => Some(Key::KEY_1),  0x1F => Some(Key::KEY_2),
        0x20 => Some(Key::KEY_3),  0x21 => Some(Key::KEY_4),
        0x22 => Some(Key::KEY_5),  0x23 => Some(Key::KEY_6),
        0x24 => Some(Key::KEY_7),  0x25 => Some(Key::KEY_8),
        0x26 => Some(Key::KEY_9),  0x27 => Some(Key::KEY_0),
        0x28 => Some(Key::KEY_ENTER),     0x29 => Some(Key::KEY_ESC),
        0x2A => Some(Key::KEY_BACKSPACE),  0x2B => Some(Key::KEY_TAB),
        0x2C => Some(Key::KEY_SPACE),      0x2D => Some(Key::KEY_MINUS),
        0x2E => Some(Key::KEY_EQUAL),      0x2F => Some(Key::KEY_LEFTBRACE),
        0x30 => Some(Key::KEY_RIGHTBRACE), 0x31 => Some(Key::KEY_BACKSLASH),
        0x33 => Some(Key::KEY_SEMICOLON),  0x34 => Some(Key::KEY_APOSTROPHE),
        0x35 => Some(Key::KEY_GRAVE),      0x36 => Some(Key::KEY_COMMA),
        0x37 => Some(Key::KEY_DOT),        0x38 => Some(Key::KEY_SLASH),
        0x39 => Some(Key::KEY_CAPSLOCK),
        0x3A => Some(Key::KEY_F1),   0x3B => Some(Key::KEY_F2),
        0x3C => Some(Key::KEY_F3),   0x3D => Some(Key::KEY_F4),
        0x3E => Some(Key::KEY_F5),   0x3F => Some(Key::KEY_F6),
        0x40 => Some(Key::KEY_F7),   0x41 => Some(Key::KEY_F8),
        0x42 => Some(Key::KEY_F9),   0x43 => Some(Key::KEY_F10),
        0x44 => Some(Key::KEY_F11),  0x45 => Some(Key::KEY_F12),
        0x46 => Some(Key::KEY_SYSRQ),     0x47 => Some(Key::KEY_SCROLLLOCK),
        0x48 => Some(Key::KEY_PAUSE),      0x49 => Some(Key::KEY_INSERT),
        0x4A => Some(Key::KEY_HOME),       0x4B => Some(Key::KEY_PAGEUP),
        0x4C => Some(Key::KEY_DELETE),     0x4D => Some(Key::KEY_END),
        0x4E => Some(Key::KEY_PAGEDOWN),   0x4F => Some(Key::KEY_RIGHT),
        0x50 => Some(Key::KEY_LEFT),       0x51 => Some(Key::KEY_DOWN),
        0x52 => Some(Key::KEY_UP),         0x53 => Some(Key::KEY_NUMLOCK),
        _ => None,
    }
}

/// Convert HID modifier byte to list of modifier keys
pub fn modifier_keys(mask: u8) -> Vec<Key> {
    let mut keys = Vec::new();
    if mask & 0x01 != 0 { keys.push(Key::KEY_LEFTCTRL); }
    if mask & 0x02 != 0 { keys.push(Key::KEY_LEFTSHIFT); }
    if mask & 0x04 != 0 { keys.push(Key::KEY_LEFTALT); }
    if mask & 0x08 != 0 { keys.push(Key::KEY_LEFTMETA); }
    if mask & 0x10 != 0 { keys.push(Key::KEY_RIGHTCTRL); }
    if mask & 0x20 != 0 { keys.push(Key::KEY_RIGHTSHIFT); }
    if mask & 0x40 != 0 { keys.push(Key::KEY_RIGHTALT); }
    if mask & 0x80 != 0 { keys.push(Key::KEY_RIGHTMETA); }
    keys
}

/// Parse key name string to evdev Key (for config bindings)
pub fn key_name_to_evdev(name: &str) -> Option<Key> {
    match name {
        "KEY_A" => Some(Key::KEY_A), "KEY_B" => Some(Key::KEY_B),
        "KEY_C" => Some(Key::KEY_C), "KEY_D" => Some(Key::KEY_D),
        "KEY_E" => Some(Key::KEY_E), "KEY_F" => Some(Key::KEY_F),
        "KEY_G" => Some(Key::KEY_G), "KEY_H" => Some(Key::KEY_H),
        "KEY_I" => Some(Key::KEY_I), "KEY_J" => Some(Key::KEY_J),
        "KEY_K" => Some(Key::KEY_K), "KEY_L" => Some(Key::KEY_L),
        "KEY_M" => Some(Key::KEY_M), "KEY_N" => Some(Key::KEY_N),
        "KEY_O" => Some(Key::KEY_O), "KEY_P" => Some(Key::KEY_P),
        "KEY_Q" => Some(Key::KEY_Q), "KEY_R" => Some(Key::KEY_R),
        "KEY_S" => Some(Key::KEY_S), "KEY_T" => Some(Key::KEY_T),
        "KEY_U" => Some(Key::KEY_U), "KEY_V" => Some(Key::KEY_V),
        "KEY_W" => Some(Key::KEY_W), "KEY_X" => Some(Key::KEY_X),
        "KEY_Y" => Some(Key::KEY_Y), "KEY_Z" => Some(Key::KEY_Z),
        "KEY_1" => Some(Key::KEY_1), "KEY_2" => Some(Key::KEY_2),
        "KEY_3" => Some(Key::KEY_3), "KEY_4" => Some(Key::KEY_4),
        "KEY_5" => Some(Key::KEY_5), "KEY_6" => Some(Key::KEY_6),
        "KEY_7" => Some(Key::KEY_7), "KEY_8" => Some(Key::KEY_8),
        "KEY_9" => Some(Key::KEY_9), "KEY_0" => Some(Key::KEY_0),
        "KEY_F1" => Some(Key::KEY_F1),   "KEY_F2" => Some(Key::KEY_F2),
        "KEY_F3" => Some(Key::KEY_F3),   "KEY_F4" => Some(Key::KEY_F4),
        "KEY_F5" => Some(Key::KEY_F5),   "KEY_F6" => Some(Key::KEY_F6),
        "KEY_F7" => Some(Key::KEY_F7),   "KEY_F8" => Some(Key::KEY_F8),
        "KEY_F9" => Some(Key::KEY_F9),   "KEY_F10" => Some(Key::KEY_F10),
        "KEY_F11" => Some(Key::KEY_F11), "KEY_F12" => Some(Key::KEY_F12),
        "KEY_ENTER" => Some(Key::KEY_ENTER),
        "KEY_ESC" => Some(Key::KEY_ESC),
        "KEY_BACKSPACE" => Some(Key::KEY_BACKSPACE),
        "KEY_TAB" => Some(Key::KEY_TAB),
        "KEY_SPACE" => Some(Key::KEY_SPACE),
        "KEY_MINUS" => Some(Key::KEY_MINUS),
        "KEY_EQUAL" => Some(Key::KEY_EQUAL),
        "KEY_LEFTBRACE" => Some(Key::KEY_LEFTBRACE),
        "KEY_RIGHTBRACE" => Some(Key::KEY_RIGHTBRACE),
        "KEY_BACKSLASH" => Some(Key::KEY_BACKSLASH),
        "KEY_SEMICOLON" => Some(Key::KEY_SEMICOLON),
        "KEY_APOSTROPHE" => Some(Key::KEY_APOSTROPHE),
        "KEY_GRAVE" => Some(Key::KEY_GRAVE),
        "KEY_COMMA" => Some(Key::KEY_COMMA),
        "KEY_DOT" => Some(Key::KEY_DOT),
        "KEY_SLASH" => Some(Key::KEY_SLASH),
        "KEY_CAPSLOCK" => Some(Key::KEY_CAPSLOCK),
        "KEY_SYSRQ" | "KEY_PRINTSCREEN" => Some(Key::KEY_SYSRQ),
        "KEY_SCROLLLOCK" => Some(Key::KEY_SCROLLLOCK),
        "KEY_PAUSE" => Some(Key::KEY_PAUSE),
        "KEY_INSERT" => Some(Key::KEY_INSERT),
        "KEY_HOME" => Some(Key::KEY_HOME),
        "KEY_PAGEUP" => Some(Key::KEY_PAGEUP),
        "KEY_DELETE" => Some(Key::KEY_DELETE),
        "KEY_END" => Some(Key::KEY_END),
        "KEY_PAGEDOWN" => Some(Key::KEY_PAGEDOWN),
        "KEY_RIGHT" => Some(Key::KEY_RIGHT),
        "KEY_LEFT" => Some(Key::KEY_LEFT),
        "KEY_DOWN" => Some(Key::KEY_DOWN),
        "KEY_UP" => Some(Key::KEY_UP),
        "KEY_NUMLOCK" => Some(Key::KEY_NUMLOCK),
        "KEY_LEFTCTRL" => Some(Key::KEY_LEFTCTRL),
        "KEY_LEFTSHIFT" => Some(Key::KEY_LEFTSHIFT),
        "KEY_LEFTALT" => Some(Key::KEY_LEFTALT),
        "KEY_LEFTMETA" => Some(Key::KEY_LEFTMETA),
        "KEY_RIGHTCTRL" => Some(Key::KEY_RIGHTCTRL),
        "KEY_RIGHTSHIFT" => Some(Key::KEY_RIGHTSHIFT),
        "KEY_RIGHTALT" => Some(Key::KEY_RIGHTALT),
        "KEY_RIGHTMETA" => Some(Key::KEY_RIGHTMETA),
        "KEY_VOLUMEUP" => Some(Key::KEY_VOLUMEUP),
        "KEY_VOLUMEDOWN" => Some(Key::KEY_VOLUMEDOWN),
        "KEY_MUTE" => Some(Key::KEY_MUTE),
        "KEY_PLAYPAUSE" => Some(Key::KEY_PLAYPAUSE),
        "KEY_NEXTSONG" => Some(Key::KEY_NEXTSONG),
        "KEY_PREVIOUSSONG" => Some(Key::KEY_PREVIOUSSONG),
        "KEY_STOPCD" => Some(Key::KEY_STOPCD),
        "KEY_MENU" => Some(Key::KEY_MENU),
        _ => None,
    }
}

/// Get all possible keys for uinput device capabilities
pub fn all_supported_keys() -> Vec<Key> {
    let mut keys = Vec::new();
    for usage in 0x04..=0x53 {
        if let Some(k) = hid_to_key(usage) { keys.push(k); }
    }
    for k in [Key::KEY_LEFTCTRL, Key::KEY_LEFTSHIFT, Key::KEY_LEFTALT, Key::KEY_LEFTMETA,
              Key::KEY_RIGHTCTRL, Key::KEY_RIGHTSHIFT, Key::KEY_RIGHTALT, Key::KEY_RIGHTMETA,
              Key::KEY_VOLUMEUP, Key::KEY_VOLUMEDOWN, Key::KEY_MUTE,
              Key::KEY_PLAYPAUSE, Key::KEY_NEXTSONG, Key::KEY_PREVIOUSSONG, Key::KEY_STOPCD,
              Key::KEY_MENU] {
        keys.push(k);
    }
    keys
}
