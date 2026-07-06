#!/usr/bin/env python3
"""
seed_test_guests.py — Insert known test guests into the SQLite database.

Creates three parties with predictable invite codes for manual testing.
Safe to run multiple times — existing test guests are replaced.

Test scenarios:
  0001  Test Solo                          — solo guest, no rehearsal
  0002  Test Alpha / Beta / Gamma          — party of 3, no rehearsal
  0003  Test Rehearsal / Test Rehearsal2   — party of 2, rehearsal invited

Usage:
  python scripts/seed_test_guests.py --db wedding.db
"""

import argparse
import sqlite3
import uuid

TEST_CODES = ("0001", "0002", "0003")

TEST_GUESTS = [
    # (first, last, email, code, rehearsal_invited)
    ("Test", "Solo",       "test.solo@example.com",      "0001", 0),
    ("Test", "Alpha",      None,                          "0002", 0),
    ("Test", "Beta",       None,                          "0002", 0),
    ("Test", "Gamma",      None,                          "0002", 0),
    ("Test", "Rehearsal",  "test.rehearsal@example.com",  "0003", 1),
    ("Test", "Rehearsal2", None,                          "0003", 1),
]


def seed(db_path: str):
    con = sqlite3.connect(db_path)

    # Remove any previous test guests so re-runs are idempotent.
    placeholders = ",".join("?" * len(TEST_CODES))
    con.execute(f"DELETE FROM rsvps WHERE guest_id IN (SELECT id FROM guests WHERE invite_code IN ({placeholders}))", TEST_CODES)
    con.execute(f"DELETE FROM guests WHERE invite_code IN ({placeholders})", TEST_CODES)

    con.executemany(
        "INSERT INTO guests (id, first_name, last_name, email, invite_code, rehearsal_invited) "
        "VALUES (?, ?, ?, ?, ?, ?)",
        [(str(uuid.uuid4()), first, last, email, code, rehearsal)
         for first, last, email, code, rehearsal in TEST_GUESTS],
    )
    con.commit()
    con.close()

    print(f"Test guests inserted into {db_path}:")
    print("  0001 — Test Solo (solo, no rehearsal)")
    print("  0002 — Test Alpha / Beta / Gamma (party of 3, no rehearsal)")
    print("  0003 — Test Rehearsal / Test Rehearsal2 (party of 2, rehearsal invited)")


def main():
    p = argparse.ArgumentParser(description="Insert test guests for manual website testing.")
    p.add_argument("--db", required=True, help="Path to the SQLite database file")
    seed(p.parse_args().db)


if __name__ == "__main__":
    main()
