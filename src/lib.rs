#![deny(clippy::all)]
#![allow(unused)]
use napi_derive::napi;
use std::collections::HashMap;
use sysinfo::{Components, Disks, Networks, System};
use windows::Win32::UI::WindowsAndMessaging::{
  GetForegroundWindow, GetWindowModuleFileNameW, GetWindowThreadProcessId,
};
use winreg::enums::HKEY_CLASSES_ROOT;
use winreg::RegKey;

const APP_NAME_SUFFIX: &str = ".FriendlyAppName";
const EXE_SUFFIX: &str = ".exe.FriendlyAppName";

fn get_all_process() -> HashMap<u32, String> {
  let mut sys = System::new_all();
  sys.refresh_all();
  let mut map: HashMap<u32, String> = HashMap::new();
  for (pid, p) in sys.processes() {
    if let Some(exe) = p.exe() {
      map.insert(
        pid.as_u32(),
        exe.to_string_lossy().to_string().to_lowercase(),
      );
    }
  }
  map
}

#[napi]
fn get_current_app_path() -> Option<String> {
  unsafe {
    let hwd = GetForegroundWindow();
    let mut pid = 0;
    GetWindowThreadProcessId(hwd, Some(&mut pid));
    let all_process = get_all_process();
    all_process.get(&pid).cloned()
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
      apps.insert(
        name.replace(APP_NAME_SUFFIX, "").to_lowercase(),
        value.to_string(),
      );
    }
  }
  apps
}
