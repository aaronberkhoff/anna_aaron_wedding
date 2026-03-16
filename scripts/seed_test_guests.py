#!/usr/bin/env python3
"""
seed_test_guests.py — Insert random test guests into the SQLite database.

Usage:
  python scripts/seed_test_guests.py --db wedding.db --count 20
  python scripts/seed_test_guests.py --db wedding.db --count 20 --clear
"""

import argparse
import random
import sqlite3
import uuid

FIRST_NAMES = [
    "James", "Mary", "Robert", "Patricia", "John", "Jennifer", "Michael", "Linda",
    "William", "Barbara", "David", "Susan", "Richard", "Jessica", "Joseph", "Sarah",
    "Thomas", "Karen", "Charles", "Lisa", "Christopher", "Nancy", "Daniel", "Betty",
    "Matthew", "Margaret", "Anthony", "Sandra", "Mark", "Ashley",
]

LAST_NAMES = [
    "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis",
    "Wilson", "Taylor", "Anderson", "Thomas", "Jackson", "White", "Harris", "Martin",
    "Thompson", "Robinson", "Clark", "Lewis", "Lee", "Walker", "Hall", "Allen",
    "Young", "Hernandez", "King", "Wright", "Lopez", "Hill",
]

DIETARY = ["none", "none", "none", "vegetarian", "vegan", "gluten_free", "halal_kosher"]


def _gen_code(used: set) -> str:
    for _ in range(10_000):
        code = f"{random.randint(0, 9999):04d}"
        if code not in used:
            used.add(code)
            return code
    raise RuntimeError("Ran out of unique invite codes")


def seed(db_path: str, count: int, clear: bool):
    con = sqlite3.connect(db_path)
    cur = con.cursor()

    if clear:
        cur.execute("DELETE FROM party_members")
        cur.execute("DELETE FROM rsvps")
        cur.execute("DELETE FROM guests")
        con.commit()
        print(f"Cleared existing guest data from {db_path}.")

    # Collect existing codes so we don't collide.
    existing_codes = {row[0] for row in cur.execute("SELECT invite_code FROM guests WHERE invite_code IS NOT NULL")}

    inserted = 0
    for _ in range(count):
        first = random.choice(FIRST_NAMES)
        last = random.choice(LAST_NAMES)
        email = f"{first.lower()}.{last.lower()}{random.randint(1, 99)}@example.com"
        code = _gen_code(existing_codes)
        rehearsal = 1 if random.random() < 0.3 else 0
        dietary = random.choice(DIETARY)
        guest_id = str(uuid.uuid4())

        cur.execute(
            """INSERT INTO guests
                   (id, first_name, last_name, email, dietary, invite_code, rehearsal_invited)
               VALUES (?, ?, ?, ?, ?, ?, ?)""",
            (guest_id, first, last, email, dietary, code, rehearsal),
        )

        # ~50% of guests have 1–2 party members
        if random.random() < 0.5:
            pm_count = random.randint(1, 2)
            for _ in range(pm_count):
                pm_first = random.choice(FIRST_NAMES)
                pm_last = last  # same family
                cur.execute(
                    "INSERT INTO party_members (id, guest_id, name, dietary) VALUES (?, ?, ?, ?)",
                    (str(uuid.uuid4()), guest_id, f"{pm_first} {pm_last}", random.choice(DIETARY)),
                )

        inserted += 1

    con.commit()
    con.close()
    print(f"Inserted {inserted} test guests into {db_path}.")


def main():
    p = argparse.ArgumentParser(description="Seed test guests into the wedding database.")
    p.add_argument("--db", required=True, help="Path to the SQLite database file")
    p.add_argument("--count", type=int, default=10, help="Number of guests to insert (default: 10)")
    p.add_argument("--clear", action="store_true", help="Delete all existing guest data first")
    args = p.parse_args()
    seed(args.db, args.count, args.clear)


if __name__ == "__main__":
    main()
