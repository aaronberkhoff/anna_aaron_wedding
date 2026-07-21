#!/usr/bin/env python3
"""
sync_emails_to_prod.py — Push guest email/rehearsal updates to the production DB
WITHOUT touching RSVPs.

Production (Fly.io volume) is the source of truth for RSVP data; the local
wedding.db is the source of truth for guest contact info (emails added to the
CSV over time). This script merges the two safely:

  1. Wakes the Fly machine and downloads /data/wedding.db AND its -wal file
     (recent RSVPs live in the WAL — skipping it would silently lose them).
  2. Opens the prod copy locally (SQLite replays the WAL automatically).
  3. Applies only guest-info changes from the local DB:
       - UPDATE email / rehearsal_invited where they differ
       - INSERT guests that exist locally but not in prod (new invites)
     Matching is by invite_code + first_name + last_name.
     rsvp_status and the rsvps table are NEVER modified.
  4. Checkpoints to a single file and uploads it back, then restarts the app.

WARNING: any RSVP submitted between download and restart is lost. The window
is under a minute — run this at a quiet time.

Usage:
  python scripts/sync_emails_to_prod.py --db wedding.db --dry-run
  python scripts/sync_emails_to_prod.py --db wedding.db
"""

import argparse
import sqlite3
import subprocess
import sys
import tempfile
import urllib.request
import uuid
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


def merge(local_db: str, prod_db: Path, dry_run: bool) -> tuple[int, int]:
    """Apply local guest-info changes to the prod copy. Returns (updated, inserted)."""
    local = sqlite3.connect(local_db)
    local.row_factory = sqlite3.Row
    prod = sqlite3.connect(prod_db)  # opening replays the WAL into the copy
    prod.row_factory = sqlite3.Row

    rsvps_before = prod.execute("SELECT count(*) FROM rsvps").fetchone()[0]
    print(f"Prod DB: {prod.execute('SELECT count(*) FROM guests').fetchone()[0]} guests, "
          f"{rsvps_before} RSVPs (preserved)")

    updated = 0
    inserted = 0

    for g in local.execute(
        "SELECT first_name, last_name, email, invite_code, rehearsal_invited FROM guests"
    ).fetchall():
        match = prod.execute(
            "SELECT id, email, invite_code, last_name, rehearsal_invited FROM guests "
            "WHERE invite_code = ? AND first_name = ? AND last_name = ?",
            (g["invite_code"], g["first_name"], g["last_name"]),
        ).fetchone()

        # Fall back to name-only match — handles a guest moved to a different
        # party (invite code changed locally). Their RSVP stays with them.
        if not match:
            by_name = prod.execute(
                "SELECT id, email, invite_code, last_name, rehearsal_invited FROM guests "
                "WHERE first_name = ? AND last_name = ?",
                (g["first_name"], g["last_name"]),
            ).fetchall()
            if len(by_name) == 1:
                match = by_name[0]
            elif len(by_name) > 1:
                print(f"  SKIP {g['first_name']} {g['last_name']}: ambiguous name match",
                      file=sys.stderr)
                continue

        # Final fallback: same party + same first name — a corrected last name.
        if not match:
            by_first = prod.execute(
                "SELECT id, email, invite_code, last_name, rehearsal_invited FROM guests "
                "WHERE invite_code = ? AND first_name = ?",
                (g["invite_code"], g["first_name"]),
            ).fetchall()
            if len(by_first) == 1:
                match = by_first[0]

        if match:
            email_changed = (match["email"] or None) != (g["email"] or None)
            code_changed = match["invite_code"] != g["invite_code"]
            name_changed = match["last_name"] != g["last_name"]
            rehearsal_changed = match["rehearsal_invited"] != g["rehearsal_invited"]
            if email_changed or code_changed or name_changed or rehearsal_changed:
                what = []
                if email_changed:
                    what.append(f"email: {match['email'] or '—'} → {g['email'] or '—'}")
                if code_changed:
                    what.append(f"party: {match['invite_code']} → {g['invite_code']}")
                if name_changed:
                    what.append(f"last name: '{match['last_name']}' → '{g['last_name']}'")
                if rehearsal_changed:
                    what.append(f"rehearsal: {match['rehearsal_invited']} → {g['rehearsal_invited']}")
                print(f"  UPDATE [{g['invite_code']}] {g['first_name']} {g['last_name']}: {', '.join(what)}")
                if not dry_run:
                    prod.execute(
                        "UPDATE guests SET last_name = ?, email = ?, invite_code = ?, "
                        "rehearsal_invited = ?, updated_at = datetime('now') WHERE id = ?",
                        (g["last_name"], g["email"], g["invite_code"],
                         g["rehearsal_invited"], match["id"]),
                    )
                updated += 1
        else:
            print(f"  INSERT [{g['invite_code']}] {g['first_name']} {g['last_name']}")
            if not dry_run:
                prod.execute(
                    "INSERT INTO guests (id, first_name, last_name, email, invite_code, rehearsal_invited) "
                    "VALUES (?, ?, ?, ?, ?, ?)",
                    (str(uuid.uuid4()), g["first_name"], g["last_name"],
                     g["email"], g["invite_code"], g["rehearsal_invited"]),
                )
            inserted += 1

    if not dry_run:
        prod.commit()
        assert prod.execute("SELECT count(*) FROM rsvps").fetchone()[0] == rsvps_before, \
            "RSVP count changed during merge — aborting"
        # Collapse WAL into the main file so we upload a single self-contained DB.
        prod.execute("PRAGMA wal_checkpoint(TRUNCATE)")

    prod.close()
    local.close()
    return updated, inserted


def push_to_prod(prod_db: Path):
    print("Uploading merged DB…")
    sftp_cmds = f"put {prod_db} /data/wedding.db.new\n"
    subprocess.run(["flyctl", "sftp", "shell", "-a", APP],
                   input=sftp_cmds.encode(), check=True)

    print("Swapping DB into place…")
    run(["flyctl", "ssh", "console", "-a", APP, "-C",
         "sh -c 'rm -f /data/wedding.db-wal /data/wedding.db-shm "
         "&& mv /data/wedding.db.new /data/wedding.db "
         "&& chown wedding:wedding /data/wedding.db'"])

    print("Restarting app…")
    run(["flyctl", "apps", "restart", APP])


def main():
    p = argparse.ArgumentParser(description="Sync guest emails from local DB to production.")
    p.add_argument("--db", required=True, help="Path to the local SQLite database")
    p.add_argument("--dry-run", action="store_true",
                   help="Show what would change without pushing anything")
    args = p.parse_args()

    if not Path(args.db).exists():
        print(f"ERROR: local database not found: {args.db}", file=sys.stderr)
        sys.exit(1)

    wake_machine()

    with tempfile.TemporaryDirectory() as tmp_str:
        tmp = Path(tmp_str)
        prod_db = download_prod(tmp)
        updated, inserted = merge(args.db, prod_db, args.dry_run)

        if updated == 0 and inserted == 0:
            print("\nNothing to sync — prod is up to date.")
            return

        if args.dry_run:
            print(f"\n[DRY RUN] Would update {updated} and insert {inserted} guests. "
                  f"Re-run without --dry-run to apply.")
            return

        push_to_prod(prod_db)
        print(f"\nDone: {updated} updated, {inserted} inserted. RSVPs untouched.")


if __name__ == "__main__":
    main()
