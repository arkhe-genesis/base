1. Implement the `cathedral-live-image` crate in `crates/cathedral-live-image/src/lib.rs` to handle bundle file format (`.arkhe`).
   - Define structure representations: `BundleHeader`, `ImageSpec`, `LayerSpec`, `LayerTableEntry`.
   - Implement `BundleReader` to parse `.arkhe` bundles, and functions `extract_layer` and `extract_all_layers` logic.
   - Implement `BundleWriter` to compress layers and create `.arkhe` bundles, generating the layer entries from provided files, calculating the BLAKE3 hashes for checking, writing the manifest and layers properly.
2. Run `cargo check -p cathedral-live-image` and `cargo test -p cathedral-live-image` to ensure the changes are correct and without compilation issues.
3. Complete pre commit steps
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
4. Submit the change.
   - I will submit the change with a descriptive commit message.
