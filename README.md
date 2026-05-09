# omni_conf 🦀

**omni_conf** is a robust, cross-platform configuration manager for Rust. It handles the "where do I put this file?" headache by automatically resolving the correct application data directories for Desktop (Windows, macOS, Linux) and Mobile (iOS, Android) platforms.

## 🚀 Features

- **True Cross-Platform**: Specialized path resolution for Windows, macOS, Linux, Android, and iOS.
- **Atomic Writes**: Prevents configuration corruption by using a "write-to-tmp-then-rename" strategy.
- **Multiple Formats**: Built-in support for **JSON** and **TOML** (via feature flags).
- **Mobile Ready**: Handles iOS `NSApplicationSupportDirectory` and Android internal data paths out of the box.
- **Type-Safe**: Seamless integration with `serde`.
- **Sanitization**: Automatically cleanses app identifiers to ensure filesystem compatibility.

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
omni_conf = "0.1.0"
serde = { version = "1.0", features = ["derive"] }
```