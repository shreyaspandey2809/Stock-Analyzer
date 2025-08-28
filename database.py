# database.py
import sqlite3
from datetime import datetime

DB_NAME = "history.db"

def create_table():
    """Create the history table if it doesn't exist."""
    conn = sqlite3.connect(DB_NAME)
    cursor = conn.cursor()
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            output_text TEXT,
            chart_path TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    """)
    conn.commit()
    conn.close()

def insert_record(output_text, chart_path=None):
    """Insert a new record into the history table."""
    conn = sqlite3.connect(DB_NAME)
    cursor = conn.cursor()
    cursor.execute("INSERT INTO history (output_text, chart_path) VALUES (?, ?)", 
                   (output_text, chart_path))
    conn.commit()
    conn.close()

def fetch_records(limit=10):
    """Fetch recent records from the history table."""
    conn = sqlite3.connect(DB_NAME)
    cursor = conn.cursor()
    cursor.execute("SELECT id, output_text, chart_path, created_at FROM history ORDER BY created_at DESC LIMIT ?", (limit,))
    rows = cursor.fetchall()
    conn.close()
    return rows

if __name__ == "__main__":
    create_table()
    print("Database initialized ✅")
