#!/usr/bin/env python3
"""
purge_guests.py — Delete all guest, party member, and RSVP data from the database.

Two confirmation prompts are required before any data is deleted.

Usage:
  python scripts/purge_guests.py --db wedding.db
"""

import argparse
import sqlite3
import sys


def confirm(prompt: str) -> bool:
    answer = input(prompt).strip().lower()
    return answer in ("yes", "y")


def purge(db_path: str):
    if not confirm("Are you sure you want to purge ALL guest data? [yes/no]: "):
        print("Aborted.")
        sys.exit(0)

    if not confirm("Are you REALLY sure? This cannot be undone. [yes/no]: "):
        print("Aborted.")
        sys.exit(0)

    con = sqlite3.connect(db_path)
    cur = con.cursor()

    cur.execute("DELETE FROM party_members")
    party_count = cur.rowcount

    cur.execute("DELETE FROM rsvps")
    rsvp_count = cur.rowcount

    cur.execute("DELETE FROM guests")
    guest_count = cur.rowcount

    con.commit()
    con.close()

    print(
        f"Purged {guest_count} guests, "
        f"{party_count} party members, "
        f"{rsvp_count} RSVPs from {db_path}."
    )


def main():
    p = argparse.ArgumentParser(description="Purge all guest data from the wedding database.")
    p.add_argument("--db", required=True, help="Path to the SQLite database file")
    args = p.parse_args()
    purge(args.db)


if __name__ == "__main__":
    main()
