# dtln-rs

dtln-rs provides near real-time noise suppression for audio.

Built on a Dual-Signal Transformation LSTM Network (DTLN) approach but designed to be lightweight and portable, this module provide an embeddable noise reduction solution. It is packaged as a small Rust project that produces a WebAssembly module, a native Rust target library, and a NodeJS native module that can be easily embedded in your clients, and interfaced with WebRTC.

## Description

This project is a noise reduction module that utilizes Rust and Node.js. It provides various scripts for building and installing the module on different platforms.

## Installation

To install the module, you can use one of the following scripts based on your platform:

- **Mac x86_64**: `npm run install-mac-x86_64`
- **Mac ARM64**: `npm run install-mac-arm64`
- **WASM**: `npm run install-wasm`
- **Native**: `npm run install-native`

## Build Steps

The following build steps are available in the `package.json`:

- **install-mac-x86_64**: Cleans the build environment and builds the project for the x86_64 architecture on macOS. It uses `cargo-cp-artifact` to copy the build artifact and renames it to `dtln.js`.

- **install-mac-arm64**: Similar to the x86_64 script, but targets the ARM64 architecture on macOS.

- **install-wasm**: Runs a Node.js script to install the WebAssembly version of the module.

- **build**: Builds the project using `cargo` with JSON-rendered diagnostics.

- **build-debug**: Runs the `build` script in debug mode.

- **build-release**: Runs the `build` script in release mode.

- **install-native**: Determines the target architecture and runs the appropriate installation script for macOS.

- **test**: Runs the test suite using `cargo test`.

## Usage

To use the dtln-rs module, follow these steps:

1. **Installation**: Follow the installation instructions above to set up the module on your platform.
2. **Running the Module**: After installation, you can run the module using the appropriate command for your platform.
3. **Configuration**: If there are any configuration files or environment variables, describe how to set them up here.

## Contributing

We welcome contributions to the dtln-rs project! If you would like to contribute, please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bugfix.
3. Make your changes and commit them with clear and concise messages.
4. Push your changes to your fork.
5. Submit a pull request to the main repository.

Please ensure that your code adheres to the project's coding standards and includes appropriate tests.

## Support

If you encounter any issues or have questions, please open an issue in the GitHub repository. We will do our best to assist you.

## Author

Jason Thomas

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
