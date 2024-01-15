#![deny(clippy::all)]
#![allow(unused)]
use napi_derive::napi;
use std::collections::HashMap;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use windows::Win32::Foundation::{HINSTANCE, MAX_PATH};
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
use winreg::enums::HKEY_CLASSES_ROOT;
use winreg::RegKey;

unsafe fn null_terminated_wchar_to_string(slice: &[u16]) -> String {
  match slice.iter().position(|&x| x == 0) {
    Some(pos) => OsString::from_wide(&slice[..pos])
      .to_string_lossy()
      .into_owned(),
    None => OsString::from_wide(slice).to_string_lossy().into_owned(),
  }
}

#[napi]
fn get_current_app_path() -> String {
  unsafe {
    let options = PROCESS_QUERY_INFORMATION | PROCESS_VM_READ;
    let hwd = GetForegroundWindow();
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwd, Some(&mut pid));
    let handle = OpenProcess(options, false, pid).unwrap_or_default();
    let mut exe_buffer = [0u16; MAX_PATH as usize + 1];
    GetModuleFileNameExW(handle, HINSTANCE::default(), exe_buffer.as_mut_slice());
    null_terminated_wchar_to_string(&exe_buffer)
  }
}

const APP_NAME_SUFFIX: &str = ".FriendlyAppName";
const EXE_SUFFIX: &str = ".exe.FriendlyAppName";
const SYSTEM_BINEXE_PERFIX: &str = "C:\\Windows";

#[napi]
fn get_cached_apps() -> HashMap<String, String> {
  let system = RegKey::predef(HKEY_CLASSES_ROOT)
    .open_subkey("Local Settings\\Software\\Microsoft\\Windows\\Shell\\MuiCache")
    .unwrap();
  let mut apps: HashMap<String, String> = HashMap::new();
  for (name, value) in system.enum_values().map(|x| x.unwrap()) {
    if name.ends_with(EXE_SUFFIX) && !name.starts_with(SYSTEM_BINEXE_PERFIX) {
      apps.insert(
        name.replace(APP_NAME_SUFFIX, "").to_lowercase(),
        value.to_string(),
      );
    }
  }
  apps
}
