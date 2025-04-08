import sqlite3
import sys

# Connect to the SQLite database (it will be created if it doesn't exist)
conn = sqlite3.connect(sys.argv[1] if len(sys.argv) > 1 else 'example.db')
cursor = conn.cursor()

# Create a table

items = [
        (1, 1, 1, "9.8 Action Comics #1"),
        (1, 1, 1, "9.7 Amazing Fantasy #15"),
        (1, 1, 1, "9.0 Detective Comics #27"),
        (1, 1, 2, "Inu Sakuya Izayoi"),
        (1, 1, 3, "Stan Lee Funko Pop!"),
        (1, 1, 4, "Duck Tales 2"),
        (1, 1, 4, "Power Blade 2"),
        (1, 1, 4, "Bubble Bobble 2"),
]

# Insert data into the table
cursor.executemany('''
INSERT OR IGNORE INTO Item (rack_id, shelf_id, basket_id, name)
VALUES (?, ?, ?, ?)
''', items)

# Commit changes and close the connection
conn.commit()
conn.close()

print("Database filled successfully.")

