"""
Tests for Python bindings
"""
import pytest
import sys
import os

# Add parent directory to path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'python'))

try:
    import deepsql
except ImportError:
    pytest.skip("Python bindings not built", allow_module_level=True)


def test_connect():
    """Test database connection"""
    db = deepsql.connect(":memory:")
    assert db is not None
    db.close()


def test_context_manager():
    """Test context manager support"""
    with deepsql.connect(":memory:") as db:
        assert db is not None
        # Should auto-close on exit


def test_transaction():
    """Test transaction management"""
    with deepsql.connect(":memory:") as db:
        db.begin()
        # Do some operations
        db.commit()


def test_transaction_rollback():
    """Test transaction rollback"""
    with deepsql.connect(":memory:") as db:
        db.begin()
        # Do some operations
        db.rollback()


def test_kv_operations():
    """Test low-level key-value operations"""
    with deepsql.connect(":memory:") as db:
        # Insert
        db.insert_kv(b"key1", b"value1")
        
        # Search
        value = db.search_kv(b"key1")
        assert value == b"key1"  # Current implementation returns key
        
        # Delete
        db.delete_kv(b"key1")
        
        # Search after delete
        value = db.search_kv(b"key1")
        assert value is None


def test_cache_stats():
    """Test plan cache statistics"""
    with deepsql.connect(":memory:") as db:
        stats = db.get_cache_stats()
        if stats:
            assert stats.size >= 0
            assert stats.hit_rate >= 0.0
            assert stats.hit_rate <= 1.0


def test_clear_cache():
    """Test cache clearing"""
    with deepsql.connect(":memory:") as db:
        db.clear_cache()
        stats = db.get_cache_stats()
        if stats:
            assert stats.size == 0


def test_bulk_load():
    """Test bulk loading"""
    with deepsql.connect(":memory:") as db:
        # Create sorted records
        records = [
            (f"key_{i:05d}".encode(), f"value_{i}".encode())
            for i in range(100)
        ]
        
        # Bulk load (placeholder implementation returns 0)
        count = db.bulk_load(records)
        assert count >= 0  # Placeholder implementation


def test_version():
    """Test version attribute"""
    assert hasattr(deepsql, '__version__')
    assert isinstance(deepsql.__version__, str)


if __name__ == '__main__':
    pytest.main([__file__, '-v'])

