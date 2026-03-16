#!/usr/bin/env python3
"""
import_guests.py — Load wedding guests from an Excel spreadsheet into SQLite.

Expected spreadsheet columns (case-insensitive):
  Name              – Primary guest full name (required)
  Email             – Email address (optional)
  InviteToRehearsal – "true" / "false" / "yes" / "no" (optional, default false)
  InviteCode        – 4-digit code string (optional; generated if blank)
  Guest1, Guest2…   – Party member names (optional, any number of columns)

Usage:
  python scripts/import_guests.py guests.xlsx --db wedding.db
  python scripts/import_guests.py guests.xlsx --db wedding.db --send-invites
  python scripts/import_guests.py guests.xlsx --db wedding.db --dry-run

Dependencies:
  pip install openpyxl
  pip install openpyxl  (for --send-invites: smtplib is in stdlib)
"""

import argparse
import os
import random
import smtplib
import sqlite3
import sys
import uuid
from email.mime.text import MIMEText
from pathlib import Path

try:
    import openpyxl
except ImportError:
    print("ERROR: openpyxl is required. Install with: pip install openpyxl", file=sys.stderr)
    sys.exit(1)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _truthy(val: str) -> bool:
    return str(val).strip().lower() in ("true", "yes", "1", "y")


def _normalize_header(h: str) -> str:
    return str(h).strip().lower().replace(" ", "").replace("_", "")


def _generate_code(used: set) -> str:
    """Return a unique random 4-digit string not already in `used`."""
    for _ in range(10_000):
        code = f"{random.randint(0, 9999):04d}"
        if code not in used:
            used.add(code)
            return code
    raise RuntimeError("Could not generate a unique invite code after 10 000 attempts")


def _send_invite_email(smtp_cfg: dict, to_email: str, guest_name: str, code: str, base_url: str):
    link = f"{base_url}/rsvp?code={code}"
    body = (
        f"Dear {guest_name},\n\n"
        f"You are cordially invited to the wedding of Anna & Aaron!\n\n"
        f"Please RSVP using your personal link:\n"
        f"  {link}\n\n"
        f"Your invite code: {code}\n\n"
        f"We look forward to celebrating with you.\n\n"
        f"With love,\n"
        f"Anna & Aaron"
    )
    msg = MIMEText(body)
    msg["Subject"] = "You're invited — Anna & Aaron's Wedding"
    msg["From"] = smtp_cfg["from"]
    msg["To"] = to_email

    with smtplib.SMTP_SSL("smtp.gmail.com", 465) as server:
        server.login(smtp_cfg["username"], smtp_cfg["password"])
        server.sendmail(smtp_cfg["from"], [to_email], msg.as_string())


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def parse_args():
    p = argparse.ArgumentParser(description="Import wedding guests from Excel into SQLite.")
    p.add_argument("xlsx", help="Path to the Excel spreadsheet")
    p.add_argument("--db", required=True, help="Path to the SQLite database file")
    p.add_argument("--send-invites", action="store_true",
                   help="Send invite emails to guests with an email address")
    p.add_argument("--base-url", default="https://anna-aaron-wedding.fly.dev",
                   help="Base URL for the RSVP invite link (default: production URL)")
    p.add_argument("--dry-run", action="store_true",
                   help="Print what would be inserted without modifying the database")
    return p.parse_args()


def load_spreadsheet(path: str):
    """Return a list of row dicts from the first sheet."""
    wb = openpyxl.load_workbook(path, data_only=True)
    ws = wb.active

    rows = list(ws.iter_rows(values_only=True))
    if not rows:
        print("ERROR: spreadsheet is empty", file=sys.stderr)
        sys.exit(1)

    raw_headers = [str(h) if h is not None else "" for h in rows[0]]
    norm_headers = [_normalize_header(h) for h in raw_headers]

    records = []
    for row in rows[1:]:
        if all(v is None for v in row):
            continue  # skip blank rows
        record = {norm_headers[i]: (row[i] if row[i] is not None else "") for i in range(len(raw_headers))}
        records.append(record)

    return norm_headers, records


def build_guests(norm_headers, records):
    """Parse spreadsheet rows into a list of guest dicts ready for DB insertion."""
    used_codes = set()
    guests = []

    for record in records:
        name = str(record.get("name", "")).strip()
        if not name:
            continue

        parts = name.split(None, 1)
        first = parts[0]
        last = parts[1] if len(parts) > 1 else ""

        email = str(record.get("email", "")).strip() or None
        rehearsal = _truthy(record.get("invitetorehearsaldinner", record.get("invitetorehearsal", "")))
        code_raw = str(record.get("invitecode", "")).strip()
        invite_code = code_raw if code_raw else _generate_code(used_codes)

        if invite_code:
            used_codes.add(invite_code)

        # Collect party members from any GuestN columns
        party = []
        for i in range(1, 50):
            key = f"guest{i}"
            if key not in norm_headers:
                break
            pm_name = str(record.get(key, "")).strip()
            if pm_name:
                party.append(pm_name)

        guests.append({
            "id": str(uuid.uuid4()),
            "first_name": first,
            "last_name": last,
            "email": email,
            "invite_code": invite_code,
            "rehearsal_invited": 1 if rehearsal else 0,
            "party_members": party,
        })

    return guests


def run_import(db_path: str, guests: list, dry_run: bool):
    """Insert guests and party members into SQLite. Returns the final guest list with their codes."""
    if dry_run:
        print(f"\n[DRY RUN] Would insert {len(guests)} primary guests:")
        for g in guests:
            pm_names = ", ".join(g["party_members"]) or "—"
            rehearsal_flag = "✓ rehearsal" if g["rehearsal_invited"] else ""
            print(f"  {g['first_name']} {g['last_name']} (code: {g['invite_code']}) {rehearsal_flag}")
            if g["party_members"]:
                print(f"    Party: {pm_names}")
        return guests

    con = sqlite3.connect(db_path)
    cur = con.cursor()

    inserted = 0
    skipped = 0

    for g in guests:
        try:
            cur.execute(
                """INSERT INTO guests
                       (id, first_name, last_name, email, invite_code, rehearsal_invited)
                   VALUES (?, ?, ?, ?, ?, ?)
                   ON CONFLICT(email) DO UPDATE SET
                       invite_code = excluded.invite_code,
                       rehearsal_invited = excluded.rehearsal_invited""",
                (g["id"], g["first_name"], g["last_name"], g["email"],
                 g["invite_code"], g["rehearsal_invited"]),
            )
            guest_db_id = g["id"]

            for pm_name in g["party_members"]:
                cur.execute(
                    "INSERT INTO party_members (id, guest_id, name) VALUES (?, ?, ?)",
                    (str(uuid.uuid4()), guest_db_id, pm_name),
                )
            inserted += 1
        except sqlite3.IntegrityError as e:
            print(f"  SKIP {g['first_name']} {g['last_name']}: {e}", file=sys.stderr)
            skipped += 1

    con.commit()
    con.close()
    print(f"Imported {inserted} guests ({skipped} skipped). DB: {db_path}")
    return guests


def send_invites(guests: list, smtp_cfg: dict, base_url: str, dry_run: bool):
    sent = 0
    skipped = 0
    for g in guests:
        if not g.get("email"):
            skipped += 1
            continue
        name = f"{g['first_name']} {g['last_name']}"
        if dry_run:
            print(f"  [DRY RUN] Would send invite to {name} <{g['email']}> (code: {g['invite_code']})")
            sent += 1
            continue
        try:
            _send_invite_email(smtp_cfg, g["email"], name, g["invite_code"], base_url)
            print(f"  Sent invite to {name} <{g['email']}>")
            sent += 1
        except Exception as exc:
            print(f"  FAILED to send to {g['email']}: {exc}", file=sys.stderr)
    print(f"Invite emails: {sent} sent, {skipped} skipped (no email).")


def main():
    args = parse_args()

    if not Path(args.xlsx).exists():
        print(f"ERROR: file not found: {args.xlsx}", file=sys.stderr)
        sys.exit(1)

    print(f"Loading {args.xlsx}…")
    norm_headers, records = load_spreadsheet(args.xlsx)
    print(f"  Found {len(records)} data rows, columns: {', '.join(norm_headers)}")

    guests = build_guests(norm_headers, records)
    print(f"  Parsed {len(guests)} guests.")

    guests = run_import(args.db, guests, dry_run=args.dry_run)

    if args.send_invites:
        smtp_cfg = {
            "from": os.environ.get("SMTP_FROM", ""),
            "username": os.environ.get("SMTP_USERNAME", ""),
            "password": os.environ.get("SMTP_PASSWORD", ""),
        }
        missing = [k for k, v in smtp_cfg.items() if not v]
        if missing:
            print(f"ERROR: missing SMTP env vars: {', '.join('SMTP_' + k.upper() for k in missing)}",
                  file=sys.stderr)
            sys.exit(1)
        print("Sending invite emails…")
        send_invites(guests, smtp_cfg, args.base_url, dry_run=args.dry_run)


if __name__ == "__main__":
    main()
