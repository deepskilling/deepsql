# Phase A - Week 1 Implementation Plan

## Goal: Get Basic SELECT Working End-to-End

Target queries:
```sql
SELECT * FROM users;
SELECT id, name FROM users WHERE age > 18;
```

## Components Needed

### 1. SQL Execution Engine (NEW)
**File**: `src/sql_engine.rs`

Responsibilities:
- Parse SQL string to AST
- Build logical plan from AST
- Optimize logical plan
- Convert to physical plan
- Generate VM opcodes
- Execute via VM
- Return results

**Status**: Creating now

### 2. Enhanced Catalog Manager
**File**: `src/catalog/manager.rs`

Enhancements needed:
- ✅ Load table schemas
- ✅ Get table metadata (columns, types)
- ✅ Get root page for table
- ⏳ Create table with schema
- ⏳ Validate schema

**Status**: Partial (needs enhancement)

### 3. Complete Physical Plan to VM Compiler
**File**: `src/planner/compiler.rs` (NEW)

Responsibilities:
- Convert PhysicalPlan to VM opcodes
- Generate TableScan instructions
- Generate Filter instructions
- Generate Project instructions
- Generate ResultRow instructions

**Status**: Creating now

### 4. Record to Value Conversion
**File**: `src/storage/record.rs`

Enhancements needed:
- Convert Record values to VM Values
- Extract specific columns from records
- Handle NULL values properly

**Status**: Needs enhancement

### 5. Integration Tests
**File**: `tests/sql_basic_execution_tests.rs`

Tests to create:
- Create table
- Insert data
- SELECT * FROM table
- SELECT columns FROM table
- SELECT with WHERE clause

**Status**: Creating now

## Implementation Order

### Step 1: Create SQL Execution Engine ✅ DOING NOW
**Time**: 1-2 hours

```rust
// src/sql_engine.rs
pub struct SqlEngine {
    catalog: CatalogManager,
    pager: Pager,
}

impl SqlEngine {
    pub fn execute(&mut self, sql: &str) -> Result<QueryResult> {
        // 1. Parse SQL
        // 2. Build logical plan
        // 3. Optimize
        // 4. Build physical plan
        // 5. Compile to opcodes
        // 6. Execute
    }
}
```

### Step 2: Enhance Catalog for SELECT ⏳ NEXT
**Time**: 1 hour

- Get table schema by name
- Get column list
- Get root page ID
- Validate table exists

### Step 3: Create Physical Plan Compiler ⏳
**Time**: 2-3 hours

Convert PhysicalPlan::TableScan to:
```
TableScan { table: "users", cursor: 0 }
Rewind { cursor: 0, jump_if_empty: END }
LOOP:
  Column { cursor: 0, column: 0, dest_register: 0 }
  Column { cursor: 0, column: 1, dest_register: 1 }
  ResultRow { registers: [0, 1] }
  Next { cursor: 0, jump_if_done: END }
  Goto { address: LOOP }
END:
Halt
```

### Step 4: Implement SELECT Execution ⏳
**Time**: 2-3 hours

- Parse SELECT
- Build logical plan (Scan + Projection)
- Optimize
- Build physical plan
- Compile to opcodes
- Execute
- Return rows

### Step 5: Add WHERE Support ⏳
**Time**: 2-3 hours

- Parse WHERE clause
- Add Filter node to logical plan
- Compile Filter to VM opcodes
- Evaluate expressions in VM

### Step 6: Integration Testing ⏳
**Time**: 2 hours

- End-to-end tests
- Python bindings test
- Demo application

## Success Criteria

By end of Week 1:
- [x] SQL execution engine created
- [ ] SELECT * FROM table working
- [ ] SELECT columns FROM table working
- [ ] SELECT with WHERE working
- [ ] Tests passing
- [ ] Demo working

## Files to Create/Modify

Creating:
- [ ] src/sql_engine.rs (NEW)
- [ ] src/planner/compiler.rs (NEW)
- [ ] tests/sql_basic_execution_tests.rs (NEW)
- [ ] examples/sql_basic_demo.rs (NEW)

Modifying:
- [ ] src/lib.rs (export sql_engine)
- [ ] src/catalog/manager.rs (enhance)
- [ ] src/storage/record.rs (value conversion)
- [ ] src/planner/physical.rs (if needed)
- [ ] src/vm/executor.rs (if needed)

## Current Status

✅ Roadmap created
🔄 Starting implementation
⏳ SQL engine creation in progress

Next: Create SqlEngine struct and execute() method

