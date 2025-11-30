"""
DeepSQL - A modern embedded SQL database

High-performance embedded database with:
- ACID transactions
- SQL support
- B+Tree storage
- Plan caching (100-1000x OLTP speedup)
- Bulk loading (10-100x faster)
- Statistics-based optimization

Example:
    >>> import deepsql
    >>> db = deepsql.connect("mydb.db")
    >>> db.execute("CREATE TABLE users (id INTEGER, name TEXT)")
    >>> db.execute("INSERT INTO users VALUES (1, 'Alice')")
    >>> rows = db.query("SELECT * FROM users")
    >>> print(rows)
    [(1, 'Alice')]
    >>> db.close()

Context Manager:
    >>> with deepsql.connect("mydb.db") as db:
    ...     db.execute("INSERT INTO users VALUES (2, 'Bob')")
    ...     db.commit()
"""

__version__ = "0.1.0"

try:
    from ._deepsql import Database as _RustDatabase, CacheStats
except ImportError:
    # Fallback if Rust extension not available
    _RustDatabase = None
    CacheStats = None

from typing import List, Tuple, Any, Optional, Union
import os


class Database:
    """
    DeepSQL Database Connection
    
    High-level Python interface to DeepSQL embedded database.
    """
    
    def __init__(self, path: str):
        """
        Open or create a database
        
        Args:
            path: Path to the database file
        """
        if _RustDatabase is None:
            raise ImportError("DeepSQL Rust extension not available. Install with: pip install deepsql")
        
        self._db = _RustDatabase(path)
        self._path = path
        self._in_transaction = False
    
    def execute(self, sql: str) -> int:
        """
        Execute a SQL statement
        
        Args:
            sql: SQL statement to execute
            
        Returns:
            Number of rows affected (for INSERT/UPDATE/DELETE)
        """
        return self._db.execute_update(sql)
    
    def query(self, sql: str) -> List[Tuple[Any, ...]]:
        """
        Execute a SQL query and return all rows
        
        Args:
            sql: SQL SELECT statement
            
        Returns:
            List of tuples, one per row
        """
        rows = self._db.query(sql)
        return [tuple(row) for row in rows]
    
    def query_one(self, sql: str) -> Optional[Tuple[Any, ...]]:
        """
        Execute a SQL query and return the first row
        
        Args:
            sql: SQL SELECT statement
            
        Returns:
            First row as tuple, or None if no results
        """
        rows = self.query(sql)
        return rows[0] if rows else None
    
    def begin(self):
        """Begin a transaction"""
        self._db.begin_transaction()
        self._in_transaction = True
    
    def commit(self):
        """Commit the current transaction"""
        self._db.commit()
        self._in_transaction = False
    
    def rollback(self):
        """Rollback the current transaction"""
        self._db.rollback()
        self._in_transaction = False
    
    def insert_kv(self, key: bytes, value: bytes):
        """
        Low-level key-value insert
        
        Args:
            key: Record key (bytes)
            value: Record value (bytes)
        """
        self._db.insert(list(key), list(value))
    
    def search_kv(self, key: bytes) -> Optional[bytes]:
        """
        Low-level key-value search
        
        Args:
            key: Record key (bytes)
            
        Returns:
            Record value (bytes) or None if not found
        """
        result = self._db.search(list(key))
        return bytes(result) if result else None
    
    def delete_kv(self, key: bytes):
        """
        Low-level key-value delete
        
        Args:
            key: Record key (bytes)
        """
        self._db.delete(list(key))
    
    def bulk_load(self, records: List[Tuple[bytes, bytes]]) -> int:
        """
        Bulk load records (10-100x faster than sequential inserts)
        
        Args:
            records: List of (key, value) tuples, must be sorted by key
            
        Returns:
            Number of records loaded
        """
        rust_records = [(list(k), list(v)) for k, v in records]
        return self._db.bulk_load(rust_records)
    
    def collect_statistics(self, table: str):
        """
        Collect statistics for query optimization
        
        Args:
            table: Table name
        """
        self._db.collect_statistics(table)
    
    def get_cache_stats(self) -> Optional['CacheStats']:
        """
        Get plan cache statistics
        
        Returns:
            CacheStats object with hit rate, size, etc.
        """
        return self._db.get_cache_stats()
    
    def clear_cache(self):
        """Clear the query plan cache"""
        self._db.clear_cache()
    
    def close(self):
        """Close the database connection"""
        if self._in_transaction:
            self.rollback()
        self._db.close()
    
    def __enter__(self):
        """Context manager entry"""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit"""
        if exc_type is not None:
            # Exception occurred, rollback
            if self._in_transaction:
                self.rollback()
        self.close()
        return False
    
    def __repr__(self):
        return f"Database('{self._path}')"


def connect(path: str) -> Database:
    """
    Connect to a DeepSQL database
    
    Args:
        path: Path to the database file
        
    Returns:
        Database connection object
        
    Example:
        >>> db = deepsql.connect("mydb.db")
        >>> db.execute("CREATE TABLE users (id INT, name TEXT)")
        >>> db.close()
        
        Or with context manager:
        >>> with deepsql.connect("mydb.db") as db:
        ...     db.execute("INSERT INTO users VALUES (1, 'Alice')")
    """
    return Database(path)


# Export main classes and functions
__all__ = [
    'Database',
    'connect',
    'CacheStats',
    '__version__',
]

