// High level wrapper around DTLN that provides a simple interface.

use anyhow::{Context, Result};
use neon::prelude::*;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use crate::dtln_engine::{dtln_create, dtln_denoise, DtlnEngine};

// The main interface trait that all processors must implement.
pub trait DtlnProcessEngine<T> {
    #[allow(clippy::new_ret_no_self)]
    fn new() -> Result<T>;
    fn denoise(&mut self, input: &[f32]) -> Result<DenoiseResult>;
    fn stop(&mut self);
}

// A processor which defers processing to a separate thread.
// This allows the caller to have a non-blocking interface.
pub struct DtlnDeferredProcessor {
    pub engine: Arc<Mutex<DtlnEngine>>,
    sender_to_processor: Mutex<mpsc::Sender<Vec<f32>>>,
    receiver_from_processor: Mutex<mpsc::Receiver<std::result::Result<Vec<f32>, String>>>,
    processing: Arc<AtomicBool>,
    first_sample: AtomicBool,
    processor_handle: Option<thread::JoinHandle<()>>,
}

impl Finalize for DtlnDeferredProcessor {
    fn finalize<'a, C: neon::prelude::Context<'a>>(self, _: &mut C) {
        drop(self);
    }
}

pub struct DenoiseResult {
    pub samples: Vec<f32>,
    pub processor_starved: bool,
}

pub struct DtlnImmediateProcessor {
    pub engine: DtlnEngine,
}

impl DtlnProcessEngine<DtlnImmediateProcessor> for DtlnImmediateProcessor {
    fn new() -> Result<DtlnImmediateProcessor> {
        Ok(DtlnImmediateProcessor {
            engine: dtln_create().context("Unable to create engine.")?,
        })
    }
    fn denoise(&mut self, input: &[f32]) -> Result<DenoiseResult> {
        let mut output = vec![0.0; input.len()];
        dtln_denoise(&mut self.engine, input, &mut output)?;
        Ok(DenoiseResult {
            samples: output,
            processor_starved: false,
        })
    }

    fn stop(&mut self) {
        // NOP
    }
}

impl DtlnDeferredProcessor {
    /** If we don't already have a sample ready, and this is the first call, just return a silent
     * buffer. If we can process the input signal in real time, this means the next frame will
     * contain the denoised signal from the previous sample frame. And the next call will contain
     * the next sample frame, and so on and so forth. Otherwise we will receive an empty signal.
     *
     * In the case that the thread cannot process the input in real-time, this method will return
     * false indicating that the processor is starved.
     *
     * I.E: denoise(A) -> nothing pending -> return [0..] -> denoise(B) -> return denoised A ->
     * denoise(C) -> return denoised B -> denoise(D) -> return denoised C -> ...
     */
    fn receive_from_processor(&mut self, samples_len: usize) -> DenoiseResult {
        let max_sample_retrieval_ms = ((1000.0 / (16000.0 / samples_len as f32)) - 1.0) as u64;

        let response = self
            .receiver_from_processor
            .lock()
            .unwrap()
            .recv_timeout(std::time::Duration::from_millis(max_sample_retrieval_ms));
        let result = match response {
            Ok(processor_result) => match processor_result {
                Ok(samples) => DenoiseResult {
                    samples,
                    processor_starved: false,
                },
                Err(error) => {
                    // We can't process samples at all, it produced an error result.
                    panic!("Error in processor: {}", error);
                }
            },
            Err(_) => {
                DenoiseResult {
                    // Could be the first sample, or the processor is starved.
                    samples: vec![0.0; samples_len],
                    processor_starved: !self.first_sample.load(std::sync::atomic::Ordering::SeqCst),
                }
            } // Silence
        };

        if self.first_sample.load(std::sync::atomic::Ordering::SeqCst) {
            self.first_sample
                .store(false, std::sync::atomic::Ordering::SeqCst);
        }

        result
    }
}

impl DtlnProcessEngine<DtlnDeferredProcessor> for DtlnDeferredProcessor {
    fn new() -> Result<DtlnDeferredProcessor> {
        let engine = Arc::new(Mutex::new(
            dtln_create().context("Unable to create engine")?,
        ));
        let processing = Arc::new(AtomicBool::new(true));
        let processing_clone = processing.clone();

        let in_channel = mpsc::channel();
        let out_channel = mpsc::channel();

        let processor_receiver: Receiver<Vec<f32>> = in_channel.1;
        let sender_to_processor = in_channel.0;

        let receiver_from_processor = out_channel.1;
        let sender_from_processor = out_channel.0;

        let engine_clone = engine.clone();
        let processor_handle = thread::spawn(move || {
            while processing_clone.load(std::sync::atomic::Ordering::SeqCst) {
                // Block until samples are ready.
                let result = processor_receiver.recv();
                match result {
                    Ok(samples) => {
                        let mut out_samples = vec![0.0; samples.len()];
                        let mut engine = engine_clone.lock().unwrap();
                        let result = dtln_denoise(&mut engine, &samples, &mut out_samples);
                        match result {
                            Ok(_) => {
                                sender_from_processor.send(Ok(out_samples)).unwrap();
                            }
                            Err(dtln_error) => {
                                sender_from_processor
                                    .send(Err(dtln_error.to_string()))
                                    .unwrap();
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error in processor thread: {}", e);
                        processing_clone.store(false, std::sync::atomic::Ordering::SeqCst);
                    }
                }
            }
        });

        Ok(DtlnDeferredProcessor {
            engine,
            sender_to_processor: Mutex::new(sender_to_processor),
            receiver_from_processor: Mutex::new(receiver_from_processor),
            processing,
            first_sample: AtomicBool::new(true),
            processor_handle: Some(processor_handle),
        })
    }

    /**
     * Stops the processor thread. This will leave DtlnProcessor in a
     * state where it will always produce a poisoned result.
     *
     * This should be called when we are done with the processor.
     */
    fn stop(&mut self) {
        self.processing
            .store(false, std::sync::atomic::Ordering::SeqCst);

        // Trigger the processor to stop by sending an empty frame,
        // and having the atomic processing variable set to false.
        self.sender_to_processor
            .lock()
            .unwrap()
            .send(vec![])
            .unwrap();

        // Wait for the processor to stop.
        if let Some(processor_handle) = self.processor_handle.take() {
            processor_handle.join().unwrap();
        }
    }

    /**
     * Adds the provided samples to our processing pipeline. Returns
     * a denoised sample. If there is backflow from the processor, it will
     * be thrown away.
     *
     * This is designed for real-time processing, and samples should not be provided
     * faster than the processor can process them.
     *
     * # Arguments
     *
     * * `samples` - The samples to process. Should be uniform in size, and 16khz.
     *
     * # Returns
     * (denoised_samples: Vec<f32>, is_processor_starved: bool)
     */
    fn denoise(&mut self, samples: &[f32]) -> Result<DenoiseResult> {
        // Get converted samples for last frame from processor, if they exist.
        let processor_result = self.receive_from_processor(samples.len());

        // Send processed frame.
        match self
            .sender_to_processor
            .lock()
            .unwrap()
            .send(samples.to_vec())
        {
            Ok(_) => {}
            Err(e) => {
                // We can't process samples at all, it produced an error result.
                panic!("Error sending to processor: {}", e);
            }
        }

        Ok(processor_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deferred_denoise() -> Result<()> {
        let processor = DtlnDeferredProcessor::new();
        // Make vector 16000 long of floats between 0 and 1.
        let samples = (0..16384)
            .map(|_| rand::random::<f32>() * 10.0)
            .collect::<Vec<f32>>();
        let mut processor = processor?;

        let result = processor.denoise(&samples)?;

        // First sample shouldn't be starved, and should be silent.
        assert_eq!(result.samples, vec![0.0; result.samples.len()]);
        assert_eq!(result.processor_starved, false);

        let result = processor.denoise(&samples)?;

        // Assert original output samples length is less than or equal to input samples length.
        assert!(result.samples.len() <= samples.len());
        assert_eq!(result.processor_starved, false);

        // Assert the result is not silence (should be the result of the previous call).
        assert_ne!(result.samples, vec![0.0; result.samples.len()]);

        // Process frames 50 times, and make sure the result matches our frame size.
        for _ in 0..50 {
            let result = processor.denoise(&samples)?;
            assert_eq!(result.processor_starved, false);
            // Assert samples fit
            assert!(result.samples.len() == samples.len());
        }
        Ok(())
    }

    #[test]
    pub fn test_immediate_denoise() -> Result<()> {
        let processor = DtlnImmediateProcessor::new();
        // Make vector 16000 long of floats between 0 and 1.
        let samples = (0..16384)
            .map(|_| rand::random::<f32>() * 10.0)
            .collect::<Vec<f32>>();
        let mut processor = processor?;
        let result = processor.denoise(&samples)?;

        // Assert original output samples length is less than or equal to input samples length.
        assert!(result.samples.len() <= samples.len());
        assert_eq!(result.processor_starved, false);

        // Assert the result is not silence
        assert_ne!(result.samples, vec![0.0; result.samples.len()]);

        // Process frames 50 times, and make sure the result matches our frame size.
        for _ in 0..50 {
            let result = processor.denoise(&samples)?;
            assert_eq!(result.processor_starved, false);
            // Assert samples fit
            assert!(result.samples.len() == samples.len());
        }
        Ok(())
    }
}
