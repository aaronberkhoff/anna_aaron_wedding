#!/usr/bin/env python3
"""
clear_test_guests.py — Remove test guests (invite codes 0001, 0002, 0003) from the database.

Usage:
  python scripts/clear_test_guests.py --db wedding.db
"""

import argparse
import sqlite3

TEST_CODES = ("0001", "0002", "0003")


def clear(db_path: str):
    con = sqlite3.connect(db_path)
    placeholders = ",".join("?" * len(TEST_CODES))

    rsvps = con.execute(
        f"DELETE FROM rsvps WHERE guest_id IN (SELECT id FROM guests WHERE invite_code IN ({placeholders}))",
        TEST_CODES,
    ).rowcount
    guests = con.execute(
        f"DELETE FROM guests WHERE invite_code IN ({placeholders})",
        TEST_CODES,
    ).rowcount

    con.commit()
    con.close()
    print(f"Removed {guests} test guest(s) and {rsvps} test RSVP(s) from {db_path}.")


def main():
    p = argparse.ArgumentParser(description="Remove test guests from the wedding database.")
    p.add_argument("--db", required=True, help="Path to the SQLite database file")
    clear(p.parse_args().db)


if __name__ == "__main__":
    main()
