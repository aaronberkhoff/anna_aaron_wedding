#!/usr/bin/env python3
"""
send_invites.py — Send RSVP invite emails to guests from the SQLite database.

Every guest with an email address gets their own invite. The email includes the
shared invite code so anyone in the party can use it to RSVP for the whole group.
After a successful send, the guest's invite_sent flag is set to 1 so re-runs skip
already-contacted guests.

Use --resend to re-email guests who were already contacted.
Use --test to limit sending to test guests only (invite codes 0001/0002/0003).
Use --test-to EMAIL to route all outgoing emails to one address.

Usage:
  python scripts/send_invites.py --db wedding.db --dry-run
  python scripts/send_invites.py --db wedding.db
  python scripts/send_invites.py --db wedding.db --test --test-to you@example.com
  python scripts/send_invites.py --db wedding.db --resend

Required environment variables (when not --dry-run):
  SMTP_FROM       Sender address  (e.g. anna.aaron.wedding@gmail.com)
  SMTP_USERNAME   Gmail username  (same as SMTP_FROM for Gmail)
  SMTP_PASSWORD   Gmail app password
"""

import argparse
import os
import smtplib
import sqlite3
import sys
from email.mime.text import MIMEText
from pathlib import Path


TEST_CODES = ("0001", "0002", "0003")


def parse_args():
    p = argparse.ArgumentParser(description="Send RSVP invite emails from the SQLite database.")
    p.add_argument("--db", required=True, help="Path to the SQLite database file")
    p.add_argument("--dry-run", action="store_true",
                   help="Print what would be sent without sending anything")
    p.add_argument("--resend", action="store_true",
                   help="Re-send to guests who have already been contacted (invite_sent=1)")
    p.add_argument("--test", action="store_true",
                   help="Limit to test guests only (invite codes 0001/0002/0003)")
    p.add_argument("--test-to", metavar="EMAIL",
                   help="Route all emails to this address instead of the real recipient")
    p.add_argument("--base-url", default="https://anna-aaron-wedding.fly.dev",
                   help="Base URL for the RSVP link (default: production URL)")
    return p.parse_args()


def load_guests(db_path: str, resend: bool, test_only: bool) -> list[dict]:
    """
    Return every guest who has an email address.
    Includes their party members' names so the email can mention them.
    """
    con = sqlite3.connect(db_path)
    con.row_factory = sqlite3.Row

    # Load all guests ordered by invite_code then rowid so party members stay together.
    if test_only:
        placeholders = ",".join("?" * len(TEST_CODES))
        all_rows = con.execute(
            f"SELECT id, first_name, last_name, email, invite_code, invite_sent "
            f"FROM guests WHERE invite_code IN ({placeholders}) ORDER BY invite_code, rowid",
            TEST_CODES,
        ).fetchall()
    else:
        all_rows = con.execute(
            "SELECT id, first_name, last_name, email, invite_code, invite_sent "
            "FROM guests WHERE invite_code IS NOT NULL ORDER BY invite_code, rowid"
        ).fetchall()

    # Build a map of invite_code -> list of all member names in that party.
    party_members: dict[str, list[str]] = {}
    for row in all_rows:
        code = row["invite_code"]
        party_members.setdefault(code, []).append(f"{row['first_name']} {row['last_name']}")

    guests = []
    for row in all_rows:
        if not row["email"]:
            continue
        if row["invite_sent"] and not resend:
            continue
        guests.append({
            "id": row["id"],
            "name": f"{row['first_name']} {row['last_name']}",
            "email": row["email"],
            "invite_code": row["invite_code"],
            "party": party_members[row["invite_code"]],
        })

    con.close()
    return guests


def build_body(guest_name: str, party: list[str], code: str, base_url: str) -> str:
    link = f"{base_url}/rsvp?code={code}"

    others = [m for m in party if m != guest_name]
    if others:
        party_line = (
            f"\nThis code also covers your party: {', '.join(others)}.\n"
            f"Anyone in your group can open the link and RSVP for everyone at once.\n"
        )
    else:
        party_line = "\n"

    return (
        f"Dear {guest_name},\n"
        f"\n"
        f"You are cordially invited to the wedding of Anna & Aaron!\n"
        f"\n"
        f"Please RSVP at the link below:\n"
        f"  {link}\n"
        f"\n"
        f"Your invite code: {code}\n"
        f"{party_line}"
        f"\n"
        f"We look forward to celebrating with you.\n"
        f"\n"
        f"With love,\n"
        f"Anna & Aaron"
    )


def send_email(smtp_cfg: dict, to_email: str, subject: str, body: str):
    msg = MIMEText(body)
    msg["Subject"] = subject
    msg["From"] = smtp_cfg["from"]
    msg["To"] = to_email

    with smtplib.SMTP_SSL("smtp.gmail.com", 465) as server:
        server.login(smtp_cfg["username"], smtp_cfg["password"])
        server.sendmail(smtp_cfg["from"], [to_email], msg.as_string())


def mark_sent(db_path: str, guest_id: str):
    con = sqlite3.connect(db_path)
    con.execute("UPDATE guests SET invite_sent = 1 WHERE id = ?", (guest_id,))
    con.commit()
    con.close()


def main():
    args = parse_args()

    if not Path(args.db).exists():
        print(f"ERROR: database not found: {args.db}", file=sys.stderr)
        sys.exit(1)

    smtp_cfg = None
    if not args.dry_run:
        smtp_cfg = {
            "from": os.environ.get("SMTP_FROM", ""),
            "username": os.environ.get("SMTP_USERNAME", ""),
            "password": os.environ.get("SMTP_PASSWORD", ""),
        }
        missing = [k for k, v in smtp_cfg.items() if not v]
        if missing:
            print(
                f"ERROR: missing env vars: {', '.join('SMTP_' + k.upper() for k in missing)}",
                file=sys.stderr,
            )
            sys.exit(1)

    guests = load_guests(args.db, resend=args.resend, test_only=args.test)

    if not guests:
        suffix = " (use --resend to re-send)" if not args.resend else ""
        print(f"No guests to email{suffix}.")
        return

    mode = "[DRY RUN] " if args.dry_run else ""
    redirect = f" → {args.test_to}" if args.test_to else ""
    print(f"{mode}Sending to {len(guests)} guests{redirect}…\n")

    sent = 0
    failed = 0

    for g in guests:
        recipient = args.test_to if args.test_to else g["email"]
        others = [m for m in g["party"] if m != g["name"]]
        party_label = f" (party of {len(g['party'])})" if len(g["party"]) > 1 else ""
        print(f"  [{g['invite_code']}] {g['name']}{party_label} → {recipient}")

        if args.dry_run:
            sent += 1
            continue

        body = build_body(g["name"], g["party"], g["invite_code"], args.base_url)
        subject = "You're invited — Anna & Aaron's Wedding"

        try:
            send_email(smtp_cfg, recipient, subject, body)
            if not args.test_to:
                mark_sent(args.db, g["id"])
            sent += 1
        except Exception as exc:
            print(f"         FAILED: {exc}", file=sys.stderr)
            failed += 1

    print(f"\nDone: {sent} sent, {failed} failed.")
    if args.test_to:
        print(f"Note: invite_sent flags were NOT updated (test mode).")


if __name__ == "__main__":
    main()
