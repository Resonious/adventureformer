use std::path::Path;
use libc::{c_int, c_void, c_char};
use std::mem::transmute;
use std::ptr;
use std::ffi::CString;
use std::sync::mpsc::Sender;
use std::slice;

#[repr(C)]
struct Win32SecurityAttributes {
    length: i32, // always size_off::<Win32SecurityAttributes>()
    security_descriptor: *const c_void,
    inherit_handle:      bool
}

#[repr(C)]
struct Win32FileNotifyInformation {
    next_entry_offset: i32,
    action:            i32,
    file_name_length:  i32,
    first_file_name_char: u16
}

impl Win32FileNotifyInformation {
    pub fn file_name(&self) -> String { unsafe {
        let v = slice::from_raw_parts(&self.first_file_name_char, self.file_name_length as usize);
        String::from_utf16_lossy(v)
    }}
}

extern "C" {
    fn CreateFileA(
        file_name:            *const c_char,
        desired_access:       i32,
        share_mode:           i32,
        security_attributes:  *const Win32SecurityAttributes,
        creation_disposition: i32,
        flags_and_attributes: i32,
        template_file:        *const c_void
    ) -> *const c_void;

    fn ReadDirectoryChangesW(
        directory:          *const c_void, // Retrieved from CreateFile
        buffer:             *const c_void, // Gets dynamically filled with Win32FileNotifyInformation
        buffer_length:      i32,
        watch_subtree:      bool,
        notify_filter:      i32,
        bytes_returned:     *const i32,
        overlapped:         *const c_void,
        completion_routine: *const c_void
    ) -> i32;

    fn QueryPerformanceCounter(out: *mut i64) -> bool;
    fn QueryPerformanceFrequency(out: *mut i64) -> bool;

    fn GetLastError() -> c_int;
}
const FILE_NOTIFY_CHANGE_LAST_WRITE: i32 = 0x00000010;
const INVALID_HANDLE_VALUE: *const c_void = -1 as *const c_void;

const FILE_LIST_DIRECTORY: i32 = 1;

const FILE_SHARE_DELETE: i32 = 0x00000004;
const FILE_SHARE_READ:   i32 = 0x00000001;
const FILE_SHARE_WRITE:  i32 = 0x00000002;

const FILE_FLAG_BACKUP_SEMANTICS: i32 = 0x02000000;

const OPEN_EXISTING: i32 = 3;

pub static GAME_LIB_DIR: &'static str = "./af/target/debug/";
pub static GAME_LIB_PATH: &'static str = "./af/target/debug/af.dll";
pub static GAME_LIB_FILE: &'static str = "./af.dll";

pub fn query_performance_frequency() -> i64 {
    let mut freq = 0i64;
    unsafe {
        if !QueryPerformanceFrequency(&mut freq) {
            panic!("Couldn't query performance frequency. Error code {}.", GetLastError());
        }
    }
    freq
}
pub fn query_performance_counter(counter: &mut i64) {
    unsafe {
        if !QueryPerformanceCounter(counter) {
            panic!("Couldn't query performance counter. Error code {}.", GetLastError());
        }
    }
}

pub unsafe fn watch_for_updated_game_lib(ref sender: &Sender<()>) {
    let dylib_dir  = Path::new(GAME_LIB_DIR);

    let dylib_dir_str = CString::new(dylib_dir.to_str().unwrap()).unwrap();
    let handle = CreateFileA(
        dylib_dir_str.as_ptr(),
        FILE_LIST_DIRECTORY,
        FILE_SHARE_DELETE|FILE_SHARE_READ|FILE_SHARE_WRITE,
        ptr::null(),
        OPEN_EXISTING,
        FILE_FLAG_BACKUP_SEMANTICS,
        ptr::null()
    );
    if handle == INVALID_HANDLE_VALUE {
        match GetLastError() {
            5 => panic!("CreateFile for {} failed: Access denied", dylib_dir.display()),
            error_code => panic!("CreateFile for {} failed: Error code {}", dylib_dir.display(), error_code)
        }
    }

    let results_buffer = [0u8; 1024];
    let results_size: i32 = 0;

    loop {
        match ReadDirectoryChangesW(
            handle,
            transmute(&results_buffer[0]),
            results_buffer.len() as i32,
            false,
            FILE_NOTIFY_CHANGE_LAST_WRITE,
            &results_size,
            ptr::null(),
            ptr::null()
        ) {
            0 => println!("Failed to listen for a lib change! {}", GetLastError()),

            _ => {
                let result = transmute::<_, &Win32FileNotifyInformation>(&results_buffer[0]);
                if result.next_entry_offset != 0 {
                    panic!("YO, there are multiple entries. Handle that shit.");
                }
                let file_name = result.file_name();

                // NOTE Windows seems to just give back garbage string sizes, so
                // this file name is 'af.dll' fused with 'af.metadata.o'
                if file_name == "af.dlladata." {
                    sender.send(()).unwrap();
                }
            }
        }
    }
}
