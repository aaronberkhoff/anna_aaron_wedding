#!/usr/bin/env python3
"""
import_guests.py — Load wedding guests from a CSV into SQLite.

CSV columns (case-insensitive):
  Name              – Person's full name (required)
  Email             – Email address (optional)
  InviteToRehearsal – "true" / "false" / "yes" / "no" (optional, default false)
  InviteCode        – 4-digit code shared by everyone in the same party (required)

Every row becomes its own guest record. People sharing the same InviteCode form
a party — any of them can enter the code on the RSVP page and see the whole group.

Usage:
  python scripts/import_guests.py data/wedding_invites.csv --db wedding.db
  python scripts/import_guests.py data/wedding_invites.csv --db wedding.db --send-invites
  python scripts/import_guests.py data/wedding_invites.csv --db wedding.db --dry-run
"""

import argparse
import csv
import os
import random
import smtplib
import sqlite3
import sys
import uuid
from email.mime.text import MIMEText
from pathlib import Path


def _truthy(val) -> bool:
    return str(val).strip().lower() in ("true", "yes", "1", "y")


def _normalize_header(h: str) -> str:
    return str(h).strip().lower().replace(" ", "").replace("_", "")


def _generate_code(used: set) -> str:
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


def parse_args():
    p = argparse.ArgumentParser(description="Import wedding guests from CSV into SQLite.")
    p.add_argument("csv", help="Path to the CSV file")
    p.add_argument("--db", required=True, help="Path to the SQLite database file")
    p.add_argument("--send-invites", action="store_true",
                   help="Send invite emails to guests with an email address")
    p.add_argument("--base-url", default="https://anna-aaron-wedding.fly.dev",
                   help="Base URL for the RSVP invite link (default: production URL)")
    p.add_argument("--dry-run", action="store_true",
                   help="Print what would be inserted without modifying the database")
    return p.parse_args()


def load_csv(path: str) -> list:
    with open(path, newline="", encoding="utf-8-sig") as f:
        reader = csv.DictReader(f)
        return [
            {_normalize_header(k): (v.strip() if v else "") for k, v in row.items()}
            for row in reader
        ]


def build_guests(rows: list) -> list:
    """Parse each CSV row into a guest dict. Every row is its own guest record."""
    used_codes: set[str] = {str(r.get("invitecode", "")).strip() for r in rows if r.get("invitecode")}
    guests = []

    for row in rows:
        name = row.get("name", "").strip()
        if not name:
            continue

        parts = name.split(None, 1)
        first = parts[0]
        last = parts[1] if len(parts) > 1 else ""

        email = row.get("email", "").strip() or None
        rehearsal = _truthy(row.get("invitetorehearsaldinner", row.get("invitetorehearsal", "")))
        code = row.get("invitecode", "").strip() or _generate_code(used_codes)

        guests.append({
            "id": str(uuid.uuid4()),
            "first_name": first,
            "last_name": last,
            "email": email,
            "invite_code": code,
            "rehearsal_invited": 1 if rehearsal else 0,
        })

    return guests


def run_import(db_path: str, guests: list, dry_run: bool) -> list:
    if dry_run:
        print(f"\n[DRY RUN] Would import {len(guests)} guests:")
        for g in guests:
            rehearsal_flag = " ✓ rehearsal" if g["rehearsal_invited"] else ""
            email_flag = f" <{g['email']}>" if g["email"] else ""
            print(f"  [{g['invite_code']}] {g['first_name']} {g['last_name']}{email_flag}{rehearsal_flag}")
        return guests

    con = sqlite3.connect(db_path)
    cur = con.cursor()

    inserted = 0
    updated = 0
    skipped = 0

    for g in guests:
        try:
            existing = cur.execute(
                "SELECT id FROM guests WHERE invite_code = ? AND first_name = ? AND last_name = ?",
                (g["invite_code"], g["first_name"], g["last_name"]),
            ).fetchone()

            if existing:
                cur.execute(
                    """UPDATE guests
                       SET email=?, rehearsal_invited=?, updated_at=datetime('now')
                       WHERE id=?""",
                    (g["email"], g["rehearsal_invited"], existing[0]),
                )
                updated += 1
            else:
                cur.execute(
                    """INSERT INTO guests
                           (id, first_name, last_name, email, invite_code, rehearsal_invited)
                       VALUES (?, ?, ?, ?, ?, ?)""",
                    (g["id"], g["first_name"], g["last_name"],
                     g["email"], g["invite_code"], g["rehearsal_invited"]),
                )
                inserted += 1
        except sqlite3.IntegrityError as e:
            print(f"  SKIP {g['first_name']} {g['last_name']}: {e}", file=sys.stderr)
            skipped += 1

    con.commit()
    con.close()
    print(f"Done: {inserted} inserted, {updated} updated, {skipped} skipped. DB: {db_path}")
    return guests


def send_invites(guests: list, smtp_cfg: dict, base_url: str, dry_run: bool):
    # Send one email per invite code (to the first guest with an email for that code).
    seen_codes: set[str] = set()
    sent = 0
    skipped = 0

    for g in guests:
        code = g["invite_code"]
        if code in seen_codes or not g.get("email"):
            if not g.get("email"):
                skipped += 1
            continue
        seen_codes.add(code)
        name = f"{g['first_name']} {g['last_name']}"
        if dry_run:
            print(f"  [DRY RUN] Would send invite to {name} <{g['email']}> (code: {code})")
            sent += 1
            continue
        try:
            _send_invite_email(smtp_cfg, g["email"], name, code, base_url)
            print(f"  Sent invite to {name} <{g['email']}>")
            sent += 1
        except Exception as exc:
            print(f"  FAILED to send to {g['email']}: {exc}", file=sys.stderr)

    print(f"Invite emails: {sent} sent, {skipped} skipped (no email).")


def main():
    args = parse_args()

    if not Path(args.csv).exists():
        print(f"ERROR: file not found: {args.csv}", file=sys.stderr)
        sys.exit(1)

    print(f"Loading {args.csv}…")
    rows = load_csv(args.csv)
    print(f"  Found {len(rows)} rows.")

    guests = build_guests(rows)
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
