// FFI Wrappers and raw interfaces to DTLN engine.
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs::File;
use std::io::Result;
use std::path::Path;
use std::sync::Mutex;
use wav::Header;

use crate::dtln_engine::{dtln_create, dtln_denoise, DtlnEngine};

pub fn write_pcm32_to_wav(samples: Vec<f32>, filename: &str, audiorate: u32) -> Result<()> {
    // Convert to s16
    let header = Header::new(wav::WAV_FORMAT_IEEE_FLOAT, 1, audiorate, 32);
    let mut writer = File::create(Path::new(filename))?;
    wav::write(header, &wav::BitDepth::ThirtyTwoFloat(samples), &mut writer)?;
    Ok(())
}

pub fn read_wav_to_pcm32(input: &str, samples: &mut Vec<f32>) -> Result<u32> {
    samples.clear();
    let mut inp_file = File::open(Path::new(input))?;

    let (header, data) = wav::read(&mut inp_file)?;
    let data = data.try_into_sixteen().unwrap();

    samples.reserve(data.len());

    // The sample clips are only 16 bit mono.
    assert_eq!(header.bits_per_sample, 16);
    assert_eq!(header.channel_count, 1);

    // Convert 16 bit pcm samples in data to 32-bit float
    for sample in data.iter() {
        let mut fsample = *sample as f32 / std::u16::MAX as f32;
        if fsample > 1.0 {
            fsample = 1.0;
        }
        if fsample < -1.0 {
            fsample = -1.0;
        }
        samples.push(fsample);
    }

    Ok(header.sampling_rate)
}

const WASM_AUDIO_BLOCK_SIZE: usize = 512;

#[allow(non_camel_case_types)]
#[repr(C)]
struct audio_buffer {
    data: [f32; WASM_AUDIO_BLOCK_SIZE],
}

static ENGINE_MAP: Lazy<Mutex<HashMap<u32, DtlnEngine>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static AUDIO_BUFFER_MAP: Lazy<Mutex<HashMap<u32, audio_buffer>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static CURRENT_MAP_ID: Lazy<Mutex<u32>> = Lazy::new(|| Mutex::new(0));

/**
 * Create a new DtlnEngine and return a unique id for it.
 */
pub fn dtln_create_global() -> u32 {
    let mut engine_map = ENGINE_MAP.lock().unwrap();
    let mut memory_map = AUDIO_BUFFER_MAP.lock().unwrap();

    let engine = dtln_create();
    let id = *CURRENT_MAP_ID.lock().unwrap();
    *CURRENT_MAP_ID.lock().unwrap() += 1;

    let Some(engine) = engine else {
        panic!("Failed to create DtlnEngine");
    };

    engine_map.insert(id, engine);
    memory_map.insert(
        id,
        audio_buffer {
            data: [0.0; WASM_AUDIO_BLOCK_SIZE],
        },
    );
    id
}

pub fn dtln_destroy_global(id: u32) {
    let mut engine_map = ENGINE_MAP.lock().unwrap();
    let mut memory_map = AUDIO_BUFFER_MAP.lock().unwrap();

    engine_map.remove(&id);
    memory_map.remove(&id);
}

pub fn dtln_get_audio_buffer_raw_ptr(id: u32) -> *const f32 {
    let mut memory_map = AUDIO_BUFFER_MAP.lock().unwrap();
    let buffer = memory_map.get_mut(&id);
    if buffer.is_none() {
        panic!("Audio buffer not found for {}", id);
    }
    let buffer = buffer.unwrap();
    buffer.data.as_ptr()
}

/**
 * Denoise a block of samples.
 * @param id The unique id of the engine to use.
 */
pub fn dtln_denoise_global(id: u32) -> Result<()> {
    let mut engine_map = ENGINE_MAP.lock().unwrap();

    let engine = engine_map.get_mut(&id);
    if engine.is_none() {
        panic!("Engine not found for {}", id);
    }
    let engine = engine.unwrap();

    let mut memory_map = AUDIO_BUFFER_MAP.lock().unwrap();
    let audio_buffer = memory_map.get_mut(&id).unwrap();

    let mut out = [0.0; WASM_AUDIO_BLOCK_SIZE];
    if dtln_denoise(engine, &audio_buffer.data, &mut out).is_ok() {
        audio_buffer.data.copy_from_slice(&out);
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to denoise",
        ))
    }
}
