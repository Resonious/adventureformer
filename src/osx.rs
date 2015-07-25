// TODO this isn't actually tested lmao
use std::sync::mpsc::Sender;
use std::mem::{uninitialized};

#[repr(C)]
struct TimebaseInfo {
    numer: u32,
    denom: u32
}

extern "C" {
    fn mach_absolute_time() -> u64;
    fn mach_timebase_info(info: &mut TimebaseInfo);
}

pub static GAME_LIB_DIR: &'static str = "./af/target/debug/";
pub static GAME_LIB_PATH: &'static str = "./af/target/debug/libaf.so";
pub static GAME_LIB_FILE: &'static str = "./libaf.so";

pub fn query_performance_counter(counter: &mut i64) {
    unsafe {
        let mut time = mach_absolute_time();
        if time > i64::max_value() as u64 {
            time -= i64::max_value() as u64;
        }

        *counter = time as i64;
    }
}

pub fn query_performance_frequency() -> i64 {
    unsafe {
        let mut info: TimebaseInfo = uninitialized();
        mach_timebase_info(&mut info);
        // timebase info gives period - we want frequency
        info.denom as i64 / info.numer as i64
    }
}

pub fn watch_for_updated_game_lib(ref sender: &Sender<()>) {
    println!("on mac - no hot code update for now!");
}
