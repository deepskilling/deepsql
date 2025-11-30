<!-- Build a zero-dependency, high-performance, embedded relational database—equivalent to SQLite—using Rust.

Target properties:
Embedded (no server)
Single-file database
ACID transactions
B+Tree storage engine
SQL support
WAL journaling
Concurrency (multi-reader, single-writer)
Minimal footprint

RustLiteDB aims to be a modern, memory-safe SQLite alternative for edge devices, WASM, embedded systems, and Rust applications requiring simple, local storage. -->

✅ RustLiteDB — Phase-Wise MVP Implementation Checklist

──────────────────────────────────────────
 PHASE 1 — Storage Engine Foundation ✅
──────────────────────────────────────────
- [x] File Format (Single-File DB)
- [x] Page Manager (Pager)
- [x] Page Types 
      - [x] Header Page
      - [x] Leaf Page
      - [x] Interior Page
      - [x] Overflow Page
- [x] Record Format (Varint Encoding)
- [x] B+Tree (Tables)
- [x] Cursor API (Seek, Scan, Insert, Delete)

──────────────────────────────────────────
 PHASE 2 — WAL + ACID Transactions ✅
──────────────────────────────────────────
- [x] WAL (Write-Ahead Log)
- [x] Transaction Commit / Rollback
- [x] WAL Checkpoint Mechanism
- [x] Crash Recovery Flow
- [x] File-Based Locking (Readers–Writer)

──────────────────────────────────────────
 PHASE 3 — SQL Engine Basics ✅
──────────────────────────────────────────
- [x] SQL Lexer
- [x] SQL Parser (AST)
- [x] AST Nodes for:
      - [x] SELECT
      - [x] INSERT
      - [x] UPDATE
      - [x] DELETE
      - [x] CREATE TABLE
- [x] Expression Tree (WHERE, ORDER BY)

──────────────────────────────────────────
 PHASE 4 — Query Planner & VM Execution ✅
──────────────────────────────────────────
- [x] Logical Plan Builder
- [x] Physical Plan Generator
- [x] Execution VM (Opcode Machine)
      - [x] TableScan
      - [x] IndexScan
      - [x] Filter
      - [x] Project
      - [x] Insert
      - [x] Delete
      - [x] Update
      - [x] ResultRow
- [x] Type System (INTEGER, TEXT, REAL, BLOB)

──────────────────────────────────────────
 PHASE 5 — Catalog & Schema Management ✅
──────────────────────────────────────────
- [x] System Catalog Tables
      - [x] tables
      - [x] columns
      - [x] indexes
- [x] CREATE TABLE Execution
- [x] Schema Loader (on DB open)
- [x] Persist Schema in Meta-BTree

──────────────────────────────────────────
 PHASE 6 — Indexing Support ✅
──────────────────────────────────────────
- [x] Secondary Index B+Tree
- [x] Unique Index Support
- [x] Planner: Index Scan Selection
- [x] Basic Optimizer Rules (Predicate Pushdown)

──────────────────────────────────────────
 PHASE 7 — SQL Execution Maturity ✅
──────────────────────────────────────────
- [x] INSERT / SELECT / UPDATE / DELETE (Full flow)
- [x] Expression Evaluator
- [x] ORDER BY Support
- [x] LIMIT/OFFSET Support
- [x] Error Handling (Parser + Engine)

──────────────────────────────────────────
 PHASE 8 — CLI Tool (rustlitedb Shell) ✅
──────────────────────────────────────────
- [x] CLI Interface
- [x] Run SQL from prompt
- [x] .tables command
- [x] .schema <table>
- [x] .open <database>
- [x] Pretty printing rows
- [x] Execute SQL from command line argument



│
├── Cargo.toml
├── README.md
│
├── src/
│   ├───────────────────────────────────────────────
│   │ PHASE 1 — Storage Engine Foundation
│   ├───────────────────────────────────────────────
│   │
│   ├── storage/
│   │   ├── pager.rs
│   │   ├── page.rs
│   │   ├── btree/
│   │   │   ├── mod.rs
│   │   │   ├── node.rs
│   │   │   ├── cursor.rs
│   │   │   ├── insert.rs
│   │   │   ├── delete.rs
│   │   │   └── search.rs
│   │   ├── record.rs
│   │   └── file_format.rs
│   │
│   ├───────────────────────────────────────────────
│   │ PHASE 2 — WAL + ACID Transactions
│   ├───────────────────────────────────────────────
│   │
│   ├── wal/
│   │   ├── wal.rs
│   │   ├── frame.rs
│   │   ├── checkpoint.rs
│   │   └── recovery.rs
│   │
│   ├───────────────────────────────────────────────
│   │ PHASE 3 — SQL Engine Basics
│   ├───────────────────────────────────────────────
│   │
│   ├── sql/
│   │   ├── lexer.rs
│   │   ├── parser.rs
│   │   ├── tokens.rs
│   │   ├── ast/
│   │   │   ├── mod.rs
│   │   │   ├── expr.rs
│   │   │   ├── select.rs
│   │   │   ├── insert.rs
│   │   │   ├── update.rs
│   │   │   ├── delete.rs
│   │   │   └── create_table.rs
│   │
│   ├───────────────────────────────────────────────
│   │ PHASE 4 — Planner & VM Execution
│   ├───────────────────────────────────────────────
│   │
│   ├── planner/
│   │   ├── logical_plan.rs
│   │   ├── physical_plan.rs
│   │   └── optimizer.rs
│   │
│   ├── vm/
│   │   ├── opcode.rs
│   │   ├── instructions.rs
│   │   ├── executor.rs
│   │   └── registers.rs
│   │
│   ├───────────────────────────────────────────────
│   │ PHASE 5 — Catalog & Schema Management
│   ├───────────────────────────────────────────────
│   │
│   ├── catalog/
│   │   ├── catalog.rs
│   │   ├── schema.rs
│   │   └── metadata_btree.rs
│   │
│   ├───────────────────────────────────────────────
│   │ PHASE 6 — Indexing Support
│   ├───────────────────────────────────────────────
│   │
│   ├── index/
│   │   ├── index_manager.rs
│   │   ├── secondary_index.rs
│   │   ├── unique_index.rs
│   │   └── planner_rules.rs
│   │
│   ├───────────────────────────────────────────────
│   │ PHASE 7 — SQL Execution Maturity
│   ├───────────────────────────────────────────────
│   │
│   ├── exec/
│   │   ├── expression_eval.rs
│   │   ├── sort.rs
│   │   ├── limit.rs
│   │   └── error.rs
│   │
│   ├───────────────────────────────────────────────
│   │ PHASE 8 — CLI Tool
│   ├───────────────────────────────────────────────
│   │
│   ├── cli/
│   │   ├── shell.rs
│   │   ├── commands.rs
│   │   └── printer.rs
│   │
│   ├── engine.rs        # Main DB Engine façade
│   └── lib.rs           # Base library exports
│
└── tests/
    ├── storage_tests.rs
    ├── wal_tests.rs
    ├── sql_parser_tests.rs
    ├── planner_tests.rs
    ├── vm_tests.rs
    ├── catalog_tests.rs
    ├── index_tests.rs
    └── cli_tests.rs
