// Define webassembly interface to the library
use dtln_rs::dtln_utilities::{
    dtln_create_global, dtln_denoise_global, dtln_destroy_global, dtln_get_audio_buffer_raw_ptr,
};

// WASM Interface/exports.
#[no_mangle]
extern "C" fn dtln_create_wasm() -> u32 {
    dtln_create_global()
}

#[no_mangle]
extern "C" fn dtln_get_audio_buffer(id: u32) -> *const f32 {
    dtln_get_audio_buffer_raw_ptr(id)
}

#[no_mangle]
extern "C" fn dtln_denoise_wasm(id: u32) {
    let _ = dtln_denoise_global(id);
}

#[no_mangle]
extern "C" fn dtln_destroy_wasm(id: u32) {
    dtln_destroy_global(id);
}
