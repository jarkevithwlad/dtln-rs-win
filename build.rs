// build.rs

#[cfg(target_os = "windows")]
fn main() {
    use std::env;
    let root_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    
    // Указываем Cargo, где искать нашу сгенерированную .lib библиотеку
    println!("cargo:rustc-link-search=native={}/tflite_lib", root_dir);

    // Указываем Cargo, какую библиотеку нужно прилинковать.
    // Название библиотеки будет tensorflowlite_c.lib
    println!("cargo:rustc-link-lib=static=tensorflowlite_c");
}

// Код для macOS оставляем без изменений
#[cfg(target_os = "macos")]
fn main() {
    // ... ваш код для macOS ...
}
