use std::sync::mpsc::Sender;
use std::mem::{uninitialized};
use libc::{c_long};

pub static GAME_LIB_DIR: &'static str = "./af/target/debug/";
pub static GAME_LIB_PATH: &'static str = "./af/target/debug/libaf.so";
pub static GAME_LIB_FILE: &'static str = "./libaf.so";

#[repr(C)]
struct Timespec {
    tv_sec: usize, // time_t
    tv_nsec: c_long
}

extern "C" {
    fn clock_getcpuclockid(pid: i32, clock_id: *mut usize) -> isize;
    fn clock_getres(clock_id: usize, res: *mut Timespec) -> isize;
    fn clock_gettime(clock_id: usize, tp: *mut Timespec) -> isize;
}

const CLOCK_MONOTOMIC_RAW: usize = 4;

pub fn watch_for_updated_game_lib(ref sender: &Sender<()>) {
    println!("on linux - no hot code update for now!");
}

pub fn query_performance_counter(counter: &mut i64) {
    unsafe {
        let mut time: Timespec = uninitialized();
        if clock_gettime(CLOCK_MONOTOMIC_RAW, &mut time) != 0 { panic!("Error retrieving clock time") }
        *counter = ((time.tv_sec * 1_000_000_000) as i64 + time.tv_nsec) as i64;
    }
}

pub fn query_performance_frequency() -> i64 {
    1_000_000_000
}
