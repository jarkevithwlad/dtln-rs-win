const DTLN_SAMPLE_BLOCK_SIZE = 512;
const DTLN_SIZEOF_FLOAT32 = 4;

// Export interface that matches the node plugin.
let DtlnPlugin = {
  dtln_create: () => {
    console.log(`Creating new DTLN ${Module}`);
    return Module._dtln_create_wasm();
  },
  dtln_destroy: (handle) => Module._dtln_destroy_wasm(handle),
  dtln_denoise: (handle, input, output) => {
    let audioBufferPtr = Module._dtln_get_audio_buffer(handle) / DTLN_SIZEOF_FLOAT32;
    Module.HEAPF32.set(input, audioBufferPtr);
    Module._dtln_denoise_wasm(handle);
    output.set(Module.HEAPF32.subarray(audioBufferPtr, audioBufferPtr + DTLN_SAMPLE_BLOCK_SIZE));
    return false;
  },
};

if (typeof module !== "undefined") {
  module.exports = DtlnPlugin;
}


Module.postRun = [() => {
  console.log(`Finished loading DTLN plugin!!!`);
  DtlnPlugin.postRun && DtlnPlugin.postRun.forEach((fn) => fn());
}];
