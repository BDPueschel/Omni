use super::{ResultAction, SearchResult};
use libloading::{Library, Symbol};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, PartialEq)]
pub enum EverythingStatus {
    Ready,
    NotInstalled,
    NotRunning,
}

type SetSearchW = unsafe extern "system" fn(*const u16);
type SetMax = unsafe extern "system" fn(u32);
type QueryW = unsafe extern "system" fn(i32) -> i32;
type GetNumResults = unsafe extern "system" fn() -> u32;
type GetResultFullPathNameW = unsafe extern "system" fn(u32, *mut u16, u32) -> u32;
type IsDBLoaded = unsafe extern "system" fn() -> i32;

static EVERYTHING_LIB: OnceLock<Option<Library>> = OnceLock::new();

fn default_dll_paths() -> Vec<String> {
    vec![
        "C:\\Program Files\\Everything\\Everything64.dll".to_string(),
        "C:\\Program Files (x86)\\Everything\\Everything64.dll".to_string(),
    ]
}

pub struct EverythingProvider;

impl EverythingProvider {
    fn load_library() -> &'static Option<Library> {
        EVERYTHING_LIB.get_or_init(|| {
            for path in default_dll_paths() {
                if Path::new(&path).exists() {
                    if let Ok(lib) = unsafe { Library::new(&path) } {
                        return Some(lib);
                    }
                }
            }
            None
        })
    }

    pub fn check_status_at_path(dll_path: &str) -> EverythingStatus {
        if !Path::new(dll_path).exists() {
            return EverythingStatus::NotInstalled;
        }
        EverythingStatus::NotRunning
    }

    pub fn check_status() -> EverythingStatus {
        let lib = Self::load_library();
        match lib {
            None => EverythingStatus::NotInstalled,
            Some(lib) => {
                let is_loaded: Result<Symbol<IsDBLoaded>, _> =
                    unsafe { lib.get(b"Everything_IsDBLoaded\0") };
                match is_loaded {
                    Ok(func) => {
                        if unsafe { func() } != 0 {
                            EverythingStatus::Ready
                        } else {
                            EverythingStatus::NotRunning
                        }
                    }
                    Err(_) => EverythingStatus::NotRunning,
                }
            }
        }
    }

    pub fn search(query: &str, max_results: usize) -> Vec<SearchResult> {
        let lib = match Self::load_library() {
            Some(lib) => lib,
            None => {
                return vec![SearchResult {
                    category: "Files".to_string(),
                    title: "Everything is not installed".to_string(),
                    subtitle: "Download from voidtools.com".to_string(),
                    action: ResultAction::OpenUrl {
                        url: "https://www.voidtools.com/downloads/".to_string(),
                    },
                    icon: "alert".to_string(),
                }];
            }
        };

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Self::query_everything(lib, query, max_results)
        }));

        match result {
            Ok(results) => results,
            Err(_) => vec![],
        }
    }

    fn query_everything(lib: &Library, query: &str, max_results: usize) -> Vec<SearchResult> {
        unsafe {
            let set_search: Symbol<SetSearchW> =
                lib.get(b"Everything_SetSearchW\0").unwrap();
            let set_max: Symbol<SetMax> = lib.get(b"Everything_SetMax\0").unwrap();
            let do_query: Symbol<QueryW> = lib.get(b"Everything_QueryW\0").unwrap();
            let get_num: Symbol<GetNumResults> =
                lib.get(b"Everything_GetNumResults\0").unwrap();
            let get_path: Symbol<GetResultFullPathNameW> =
                lib.get(b"Everything_GetResultFullPathNameW\0").unwrap();

            let wide: Vec<u16> = OsStr::new(query)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            set_search(wide.as_ptr());
            set_max(max_results as u32);
            do_query(1);

            let count = get_num();
            let mut paths = Vec::new();
            for i in 0..count {
                let mut buf = vec![0u16; 1024];
                get_path(i, buf.as_mut_ptr(), buf.len() as u32);
                let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
                let path = String::from_utf16_lossy(&buf[..len]);
                paths.push(path);
            }
            Self::format_results(paths)
        }
    }

    pub fn format_results(paths: Vec<String>) -> Vec<SearchResult> {
        paths
            .into_iter()
            .map(|path| {
                let filename = Path::new(&path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                SearchResult {
                    category: "Files".to_string(),
                    title: filename,
                    subtitle: path.clone(),
                    action: ResultAction::OpenFile { path },
                    icon: "file".to_string(),
                }
            })
            .collect()
    }
}
