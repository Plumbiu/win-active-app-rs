#![deny(clippy::all)]
#![allow(dead_code)]

mod exelook;
use exelook::exe_look_base64;
use napi_derive::napi;
use std::ffi::{c_void, OsString};
use std::os::windows::ffi::OsStringExt;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{BOOL, HINSTANCE, MAX_PATH};
use windows::Win32::Storage::FileSystem::{
  GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

unsafe fn null_terminated_wchar_to_string(slice: &[u16]) -> String {
  match slice.iter().position(|&x| x == 0) {
    Some(pos) => OsString::from_wide(&slice[..pos])
      .to_string_lossy()
      .into_owned(),
    None => OsString::from_wide(slice).to_string_lossy().into_owned(),
  }
}

#[derive(Debug)]
struct LangCodePage {
  pub w_language: u16,
  pub w_code_page: u16,
}

fn get_process_name_from_path(process_path: &String) -> Result<String, ()> {
  let lptstrfilename: windows::core::HSTRING = process_path.into();
  let dwlen: u32 = unsafe { GetFileVersionInfoSizeW(&lptstrfilename, Some(std::ptr::null_mut())) };
  if dwlen == 0 {
    return Err(());
  }
  let mut lpdata: Vec<u8> = vec![0u8; dwlen.try_into().unwrap()];
  let version_info_success =
    unsafe { GetFileVersionInfoW(&lptstrfilename, 0, dwlen, lpdata.as_mut_ptr().cast()).is_ok() };
  if !version_info_success {
    return Err(());
  }
  let mut lplpbuffer: *mut c_void = std::ptr::null_mut();
  let mut pulen: u32 = 0;

  let ver_query_success: BOOL = unsafe {
    VerQueryValueW(
      lpdata.as_ptr().cast(),
      w!("\\VarFileInfo\\Translation"),
      &mut lplpbuffer,
      &mut pulen,
    )
  };

  if !ver_query_success.as_bool() {
    return Err(());
  }

  let lang: &[LangCodePage] =
    unsafe { std::slice::from_raw_parts(lplpbuffer as *const LangCodePage, 1) };

  if lang.len() == 0 {
    return Err(());
  }

  let mut query_len: u32 = 0;

  let lang = lang.get(0).unwrap();
  let lang_code = format!(
    "\\StringFileInfo\\{:04x}{:04x}\\FileDescription",
    lang.w_language, lang.w_code_page
  );
  let lang_code_string: String = lang_code.to_string();
  let lang_code_ptr: *const u16 = lang_code_string
    .encode_utf16()
    .chain(Some(0))
    .collect::<Vec<_>>()
    .as_ptr();

  let lang_code: PCWSTR = PCWSTR::from_raw(lang_code_ptr);

  let mut file_description_ptr = std::ptr::null_mut();

  let file_description_query_success: BOOL = unsafe {
    VerQueryValueW(
      lpdata.as_ptr().cast(),
      lang_code,
      &mut file_description_ptr,
      &mut query_len,
    )
  };

  if !file_description_query_success.as_bool() {
    return Err(());
  }

  let file_description =
    unsafe { std::slice::from_raw_parts(file_description_ptr.cast(), query_len as usize) };
  let file_description = String::from_utf16_lossy(file_description);
  let file_description = file_description.trim_matches(char::from(0)).to_owned();

  return Ok(file_description);
}

#[napi(object)]
pub struct APP {
  pub name: String,
  pub path: String,
}

#[napi]
fn get_current_app() -> APP {
  unsafe {
    let options = PROCESS_QUERY_INFORMATION | PROCESS_VM_READ;
    let hwd = GetForegroundWindow();
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwd, Some(&mut pid));
    let handle = OpenProcess(options, false, pid).unwrap_or_default();
    let mut exe_buffer = [0u16; MAX_PATH as usize + 1];
    GetModuleFileNameExW(handle, HINSTANCE::default(), exe_buffer.as_mut_slice());
    let app_path = null_terminated_wchar_to_string(&exe_buffer);
    APP {
      name: get_process_name_from_path(&app_path).unwrap_or("".to_string()),
      path: app_path,
    }
  }
}

#[napi]
fn get_app_icon(app_path: String) -> String {
  let app_icon = exe_look_base64(app_path);
  app_icon
}
