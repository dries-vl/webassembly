# Building
    wasm-pack build --release --target web
# Size reduction: 
## step 0
    configure smallest release build options in cargo.toml: 900kb -> 376kb
## step 1
    remove @ReQwest and @anyhow: 950kb -> 580kb
    remove @image and @tobj: 580kb -> 376kb
## step 2
    install binaryen: scoop install main/binaryen
    run wasm opt to optimize the binary (-O3 for speed, -Os for size)
    wasm-opt -Os webassembly_bg.wasm -o webassembly_bg_optimized.wasm
    is done automatically if found on path: 370kb -> 370kb
## step 3
    use 'wizer' to pre-initialize wasm code that does initializing
    add this to run first method that does starts most stuff
    ```rust
    #[export_name = "wizer.initialize"]
    pub extern "C" fn init() {
        run();
    }
    ```
    wizer webassembly_bg.wasm -o initialized.wasm
    does not decrease size, but potentially improves performance
    clashes with wasm-opt (?)
## step 4
    replace all unwraps with unwrap_or_else(|_| std::process::abort())
    avoids some panicking code: 376kb -> 370kb
