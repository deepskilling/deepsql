# DeepSQL

A zero-dependency, high-performance, embedded relational database—equivalent to SQLite—using Rust.

## Features

- **Embedded**: No server required
- **Single-file database**: Simple deployment and backup
- **ACID transactions**: Data integrity guaranteed
- **B+Tree storage engine**: Efficient data organization
- **SQL support**: Standard SQL interface
- **WAL journaling**: Write-ahead logging for durability
- **Concurrency**: Multi-reader, single-writer
- **Minimal footprint**: Suitable for edge devices, WASM, and embedded systems
- **Memory-safe**: Built with Rust for safety and performance

## Status

🚧 **Under Active Development** - Phase 1: Storage Engine Foundation

### Phase 1 Progress

- [ ] File Format (Single-File DB)
- [ ] Page Manager (Pager)
- [ ] Page Types (Header, Leaf, Interior, Overflow)
- [ ] Record Format (Varint Encoding)
- [ ] B+Tree (Tables)
- [ ] Cursor API (Seek, Scan, Insert, Delete)

## Architecture

DeepSQL is a modern, memory-safe SQLite alternative designed for:
- Edge devices
- WebAssembly applications
- Embedded systems
- Rust applications requiring simple, local storage

## Getting Started

```bash
# Build the project
cargo build --release

# Run tests
cargo test

# Build documentation
cargo doc --open
```

## Project Structure

```
deepsql/
├── src/
│   ├── storage/          # Phase 1: Storage engine
│   │   ├── pager.rs      # Page management
│   │   ├── page.rs       # Page types
│   │   ├── btree/        # B+Tree implementation
│   │   ├── record.rs     # Record format
│   │   └── file_format.rs # Database file format
│   ├── wal/              # Phase 2: Write-ahead log
│   ├── sql/              # Phase 3: SQL parser
│   ├── planner/          # Phase 4: Query planning
│   ├── vm/               # Phase 4: Execution VM
│   ├── catalog/          # Phase 5: Schema management
│   ├── index/            # Phase 6: Indexing
│   ├── exec/             # Phase 7: Execution maturity
│   ├── cli/              # Phase 8: CLI tool
│   └── lib.rs            # Library exports
└── tests/                # Integration tests
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

