#![deny(clippy::all)]
#![allow(unused)]
use napi_derive::napi;
use std::collections::HashMap;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowModuleFileNameW};
use winreg::enums::HKEY_CLASSES_ROOT;
use winreg::RegKey;

const APP_NAME_SUFFIX: &str = ".FriendlyAppName";
const EXE_SUFFIX: &str = ".exe.FriendlyAppName";

#[napi]
fn get_current_app_path() -> String {
  unsafe {
    let hwd = GetForegroundWindow();
    let mut text: [u16; 512] = [0; 512];
    let file_path_len = GetWindowModuleFileNameW(hwd, &mut text);
    String::from_utf16_lossy(&text[..file_path_len as usize])
  }
}

#[napi]
fn get_cached_apps() -> HashMap<String, String> {
  let system = RegKey::predef(HKEY_CLASSES_ROOT)
    .open_subkey("Local Settings\\Software\\Microsoft\\Windows\\Shell\\MuiCache")
    .unwrap();
  let mut apps: HashMap<String, String> = HashMap::new();
  for (name, value) in system.enum_values().map(|x| x.unwrap()) {
    if name.ends_with(EXE_SUFFIX) {
      apps.insert(name.replace(APP_NAME_SUFFIX, ""), value.to_string());
    }
  }
  apps
}
