#!/usr/bin/env python3
"""
rsvp_report.py — Print a summary of current RSVP results.

Reads the guests/rsvps tables directly from a SQLite file — either a local
copy or a fresh read-only pull from the production Fly volume. Never writes
anything back (no upload, no restart), unlike sync_emails_to_prod.py.

Usage:
  python scripts/rsvp_report.py --prod
  python scripts/rsvp_report.py --db wedding.db
  python scripts/rsvp_report.py --prod --csv rsvps.csv
"""

import argparse
import csv
import sqlite3
import subprocess
import sys
import tempfile
import urllib.request
from pathlib import Path

APP = "anna-aaron-wedding"
HEALTH_URL = f"https://{APP}.fly.dev/health"


def run(cmd: list[str], **kwargs) -> subprocess.CompletedProcess:
    print(f"  $ {' '.join(cmd)}")
    return subprocess.run(cmd, check=True, **kwargs)


def wake_machine():
    print("Waking Fly machine…")
    with urllib.request.urlopen(HEALTH_URL, timeout=30) as resp:
        if resp.status != 200:
            print(f"ERROR: health check returned {resp.status}", file=sys.stderr)
            sys.exit(1)


def download_prod(tmp: Path) -> Path:
    prod_db = tmp / "prod.db"
    run(["flyctl", "sftp", "get", "/data/wedding.db", str(prod_db), "-a", APP])
    # The WAL may not exist (e.g. right after a checkpoint) — that's fine.
    try:
        run(["flyctl", "sftp", "get", "/data/wedding.db-wal", str(tmp / "prod.db-wal"), "-a", APP])
    except subprocess.CalledProcessError:
        print("  (no WAL file on server — continuing)")
    return prod_db


def parse_args():
    p = argparse.ArgumentParser(description="Print a summary of current RSVP results.")
    src = p.add_mutually_exclusive_group(required=True)
    src.add_argument("--prod", action="store_true",
                      help="Pull a fresh read-only copy of the production DB")
    src.add_argument("--db", metavar="PATH", help="Report against a local SQLite file instead")
    p.add_argument("--csv", metavar="PATH",
                   help="Also write the full per-guest RSVP list to this CSV file")
    return p.parse_args()


def print_overview(con: sqlite3.Connection):
    total_guests = con.execute("SELECT count(*) FROM guests").fetchone()[0]
    total_parties = con.execute(
        "SELECT count(DISTINCT invite_code) FROM guests WHERE invite_code IS NOT NULL"
    ).fetchone()[0]
    print("Overview")
    print(f"  {total_guests} guests invited across {total_parties} parties\n")


def print_response_breakdown(con: sqlite3.Connection):
    total = con.execute("SELECT count(*) FROM guests").fetchone()[0]
    rows = con.execute(
        "SELECT rsvp_status, count(*) FROM guests GROUP BY rsvp_status"
    ).fetchall()
    counts = dict(rows)
    print("Response Breakdown")
    for status in ("attending", "declined", "pending"):
        n = counts.get(status, 0)
        pct = (n / total * 100) if total else 0
        print(f"  {status:<10} {n:>4}  ({pct:.0f}%)")
    print()


def print_attendance(con: sqlite3.Connection):
    reception = con.execute(
        "SELECT attending_reception, count(*) FROM rsvps GROUP BY attending_reception"
    ).fetchall()
    reception_counts = {bool(k) if k is not None else None: v for k, v in reception}
    print("Reception Attendance (of guests who've responded)")
    print(f"  attending     {reception_counts.get(True, 0):>4}")
    print(f"  not attending {reception_counts.get(False, 0):>4}")
    print()

    rehearsal = con.execute(
        """SELECT r.attending_rehearsal, count(*)
           FROM rsvps r JOIN guests g ON r.guest_id = g.id
           WHERE g.rehearsal_invited = 1
           GROUP BY r.attending_rehearsal"""
    ).fetchall()
    rehearsal_counts = {bool(k) if k is not None else None: v for k, v in rehearsal}
    print("Rehearsal Attendance (of rehearsal-invited guests who've responded)")
    print(f"  attending     {rehearsal_counts.get(True, 0):>4}")
    print(f"  not attending {rehearsal_counts.get(False, 0):>4}")
    print()


def print_pending(con: sqlite3.Connection):
    rows = con.execute(
        """SELECT invite_code, first_name, last_name
           FROM guests
           WHERE rsvp_status = 'pending'
           ORDER BY invite_code, rowid"""
    ).fetchall()
    print(f"Still Pending ({len(rows)} guests)")
    if not rows:
        print("  Everyone has responded!\n")
        return
    by_code: dict[str, list[str]] = {}
    for code, first, last in rows:
        by_code.setdefault(code or "—", []).append(f"{first} {last}")
    for code, names in by_code.items():
        print(f"  [{code}] {', '.join(names)}")
    print()


def print_final_list(con: sqlite3.Connection):
    rows = con.execute(
        """SELECT invite_code, first_name, last_name, rsvp_status
           FROM guests
           ORDER BY invite_code, rowid"""
    ).fetchall()
    attending = sum(1 for _, _, _, status in rows if status == "attending")
    print(f"Final Guest List ({len(rows)} total — {attending} attending, "
          f"{len(rows) - attending} not attending)")

    last_code = None
    for code, first, last, status in rows:
        if last_code is not None and code != last_code:
            print()
        last_code = code

        if status == "attending":
            mark, note = "✓", ""
        elif status == "declined":
            mark, note = "✗", "  (declined)"
        else:
            mark, note = "✗", "  (no response yet)"

        print(f"  {mark} {first} {last}{note}  [{code or '—'}]")
    print()


def print_messages(con: sqlite3.Connection):
    rows = con.execute(
        """SELECT g.first_name, g.last_name, r.song_request, r.message
           FROM rsvps r JOIN guests g ON r.guest_id = g.id
           WHERE (r.song_request IS NOT NULL AND r.song_request != '')
              OR (r.message IS NOT NULL AND r.message != '')
           ORDER BY r.submitted_at DESC"""
    ).fetchall()
    print(f"Messages & Song Requests ({len(rows)})")
    if not rows:
        print("  None yet.\n")
        return
    for first, last, song, message in rows:
        print(f"  {first} {last}")
        if song:
            print(f"    Song: {song}")
        if message:
            print(f"    Message: {message}")
    print()


def write_csv(con: sqlite3.Connection, path: str):
    rows = con.execute(
        """SELECT g.first_name, g.last_name, g.email,
                  r.attending_reception, r.attending_rehearsal,
                  r.song_request, r.message, r.submitted_at
           FROM rsvps r JOIN guests g ON r.guest_id = g.id
           ORDER BY r.submitted_at DESC"""
    ).fetchall()
    with open(path, "w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(["first_name", "last_name", "email", "attending_reception",
                          "attending_rehearsal", "song_request", "message", "submitted_at"])
        writer.writerows(rows)
    print(f"Wrote {len(rows)} rows to {path}")


def main():
    args = parse_args()

    if args.prod:
        wake_machine()
        tmp = tempfile.TemporaryDirectory()
        db_path = str(download_prod(Path(tmp.name)))
    else:
        if not Path(args.db).exists():
            print(f"ERROR: file not found: {args.db}", file=sys.stderr)
            sys.exit(1)
        db_path = args.db

    con = sqlite3.connect(db_path)

    print()
    print_overview(con)
    print_response_breakdown(con)
    print_attendance(con)
    print_pending(con)
    print_final_list(con)
    print_messages(con)

    print("Note: dietary restrictions aren't collected by the RSVP form today, "
          "so they can't be reported here.\n")

    if args.csv:
        write_csv(con, args.csv)

    con.close()


if __name__ == "__main__":
    main()
