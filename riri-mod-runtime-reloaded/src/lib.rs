pub mod hash;
use riri_mod_tools_proc::riri_init_fn;
use riri_mod_tools_rt::logln;

#[riri_init_fn()]
fn runtime_init() {
    if let Err(e) = hash::set_executable_hash() {
        logln!(Error, "Error while trying to get executable hash: {}", e);
    }
}