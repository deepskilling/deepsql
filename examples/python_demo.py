#!/usr/bin/env python3
"""
DeepSQL Python Demo

Demonstrates Python bindings for DeepSQL embedded database.
"""

import sys
import os

# Add python module to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'python'))

try:
    import deepsql
except ImportError:
    print("❌ DeepSQL Python bindings not installed.")
    print("\nTo build and install:")
    print("  1. Install maturin: pip install maturin")
    print("  2. Build extension: maturin develop --features python")
    print("  3. Run this script again")
    sys.exit(1)


def main():
    print("╔" + "═" * 76 + "╗")
    print("║" + " " * 20 + "DeepSQL Python Demo" + " " * 37 + "║")
    print("╚" + "═" * 76 + "╝")
    print()
    
    # Create a database
    print("📊 Opening database...")
    db = deepsql.connect("demo.db")
    print(f"   Connected: {db}")
    print()
    
    # Low-level key-value operations
    print("🔑 Testing Key-Value Operations:")
    print("   Inserting key1 = value1")
    db.insert_kv(b"key1", b"value1")
    
    print("   Searching for key1...")
    result = db.search_kv(b"key1")
    print(f"   Found: {result}")
    print()
    
    # Bulk loading demo
    print("📦 Testing Bulk Loading (10-100x faster):")
    records = [
        (f"user_{i:05d}".encode(), f"User {i}".encode())
        for i in range(100)
    ]
    count = db.bulk_load(records)
    print(f"   Loaded {count} records in bulk!")
    print()
    
    # Plan cache statistics
    print("📈 Plan Cache Statistics:")
    stats = db.get_cache_stats()
    if stats:
        print(f"   {stats}")
    print()
    
    # Transaction demo
    print("💾 Testing Transactions:")
    db.begin()
    print("   Transaction started")
    db.insert_kv(b"txn_key", b"txn_value")
    print("   Inserted data in transaction")
    db.commit()
    print("   Transaction committed ✅")
    print()
    
    # Cleanup
    db.close()
    print("✅ Database closed successfully")
    print()
    
    # Context manager demo
    print("🔄 Testing Context Manager:")
    with deepsql.connect("demo2.db") as db2:
        print(f"   Connected: {db2}")
        db2.insert_kv(b"test", b"data")
        print("   Data inserted")
    print("   Auto-closed on exit ✅")
    print()
    
    # Version info
    print("ℹ️  Version Information:")
    print(f"   DeepSQL version: {deepsql.__version__}")
    print()
    
    print("╔" + "═" * 76 + "╗")
    print("║" + " " * 25 + "Demo Complete! 🎉" + " " * 34 + "║")
    print("╚" + "═" * 76 + "╝")
    
    # Cleanup demo databases
    for f in ["demo.db", "demo.db-wal", "demo.db-lock", "demo2.db", "demo2.db-wal", "demo2.db-lock"]:
        if os.path.exists(f):
            os.remove(f)


if __name__ == '__main__':
    main()

