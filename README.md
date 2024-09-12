# Zewos

Zewos is a prototype key management system (KMS) designed to work as a module for various applications.

## Features

- Simple key-value storage operations (insert, get, remove)
- Permission-based access control
- Encryption using system-generated keys
- Automatic file and folder management

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
zewos = { git = "https://github.com/oblivisheee/zewos" }
```

## Usage

Here's a basic example of how to use Zewos:

```rust
use zewos::Storage;

fn main() {
    // Initialize a new storage instance
    let mut storage = Storage::init("your_path").expect("Failed to initialize storage");

    // Insert a key-value pair
    storage.insert(
        "test".to_string().into_bytes(),
        "test2".to_string().into_bytes()
    ).expect("Failed to insert");

    // Retrieve a value
    let value = storage.get("test".to_string().into_bytes()).expect("Failed to get value");

    // Remove a key-value pair
    storage.remove("test".to_string().into_bytes()).expect("Failed to remove");
}
```

## How It Works

1. **Initialization**: When you create a `Storage` instance, Zewos automatically:
   - Generates an encryption key based on system data
   - Creates necessary folders and files
   - Sets up the KMS structure

2. **Operations**: Zewos provides basic key-value operations:
   - `insert`: Add a new key-value pair
   - `get`: Retrieve a value by its key
   - `remove`: Delete a key-value pair

3. **Security**: All operations are subject to permission checks and data encryption.

## Limitations

- This is a pre-alpha version and does not implement all methods of a standard HashMap.
- Error handling is minimal in the current version.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the [MIT License](LICENSE).

## Disclaimer

This is an early-stage prototype. USE AT YOUR OWN RISK.
