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
Use --last-call to send a final-reminder variant (updated Sept 5 deadline, urgency
note up top) to guests who have not yet RSVP'd (rsvp_status = 'pending'), regardless
of invite_sent. Does not touch the invite_sent flag.

Usage:
  python scripts/send_invites.py --db wedding.db --dry-run
  python scripts/send_invites.py --db wedding.db
  python scripts/send_invites.py --db wedding.db --test --test-to you@example.com
  python scripts/send_invites.py --db wedding.db --resend
  python scripts/send_invites.py --db wedding.db --last-call --dry-run
  python scripts/send_invites.py --db wedding.db --last-call

Required environment variables (when not --dry-run):
  SMTP_FROM       Sender address  (e.g. anna.aaron.wedding@gmail.com)
  SMTP_USERNAME   Gmail username  (same as SMTP_FROM for Gmail)
  SMTP_PASSWORD   Gmail app password
"""

import argparse
import html
import os
import smtplib
import sqlite3
import sys
from email.mime.multipart import MIMEMultipart
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
    p.add_argument("--last-call", action="store_true",
                   help="Send a final-reminder variant (Sept 5 deadline) to guests with "
                        "rsvp_status='pending', regardless of invite_sent")
    return p.parse_args()


def load_guests(db_path: str, resend: bool, test_only: bool, last_call: bool = False) -> list[dict]:
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
            f"SELECT id, first_name, last_name, email, invite_code, invite_sent, rsvp_status, "
            f"rehearsal_invited "
            f"FROM guests WHERE invite_code IN ({placeholders}) ORDER BY invite_code, rowid",
            TEST_CODES,
        ).fetchall()
    else:
        # Exclude test guests from real sends — their fake addresses would bounce.
        placeholders = ",".join("?" * len(TEST_CODES))
        all_rows = con.execute(
            f"SELECT id, first_name, last_name, email, invite_code, invite_sent, rsvp_status, "
            f"rehearsal_invited "
            f"FROM guests WHERE invite_code IS NOT NULL AND invite_code NOT IN ({placeholders}) "
            f"ORDER BY invite_code, rowid",
            TEST_CODES,
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
        if last_call:
            # Reminders go to non-responders only, irrespective of invite_sent.
            if row["rsvp_status"] != "pending":
                continue
        elif row["invite_sent"] and not resend:
            continue
        guests.append({
            "id": row["id"],
            "name": f"{row['first_name']} {row['last_name']}",
            "email": row["email"],
            "invite_code": row["invite_code"],
            "rehearsal_invited": bool(row["rehearsal_invited"]),
            "party": party_members[row["invite_code"]],
        })

    con.close()
    return guests


def _party_line_text(guest_name: str, party: list[str]) -> str:
    others = [m for m in party if m != guest_name]
    if others:
        return (
            f"\nThis code also covers your party: {', '.join(others)}.\n"
            f"Anyone in your group can open the link and RSVP for everyone at once.\n"
        )
    return ""


GOLD = "#c8a951"
CHARCOAL = "#36454f"
CHARCOAL_MUTED = "#6f7b85"
CHAMPAGNE = "#f7e7ce"
CREAM = "#fdf8f0"
IVORY = "#fffff0"

FONT_SCRIPT = "'Dancing Script', cursive"
FONT_SERIF = "'Playfair Display', Georgia, 'Times New Roman', serif"
FONT_SANS = "'Inter', Arial, Helvetica, sans-serif"


def _party_line_html(guest_name: str, party: list[str]) -> str:
    others = [m for m in party if m != guest_name]
    if not others:
        return ""
    names = ", ".join(html.escape(m) for m in others)
    return (
        f'<p style="font-family:{FONT_SANS};font-size:13px;color:{CHARCOAL_MUTED};'
        f'margin:14px 0 0;line-height:1.6;">'
        f"This code also covers your party: <strong style=\"color:{CHARCOAL};\">{names}</strong>.<br>"
        f"Anyone in your group can open the link and RSVP for everyone at once.</p>"
    )


def build_body_text(guest_name: str, party: list[str], code: str, base_url: str,
                     rehearsal_invited: bool, last_call: bool = False) -> str:
    link = f"{base_url}/rsvp?code={code}"
    deadline = "September 5, 2026" if last_call else "September 4, 2026"
    rsvp_block = (
        f"{'LAST CALL — ' if last_call else ''}Please RSVP by {deadline} at the link below:\n"
        f"  {link}\n"
        f"\n"
        f"Your invite code: {code}\n"
        f"{_party_line_text(guest_name, party)}"
    )
    urgency_note = (
        f"This is a final reminder — we have not yet received your RSVP, and the "
        f"deadline is fast approaching.\n\n"
        if last_call else ""
    )

    if rehearsal_invited:
        return (
            f"Dear {guest_name},\n"
            f"\n"
            f"{urgency_note}"
            f"You are cordially invited to celebrate the rehearsal and wedding of Anna Pauline Hagen "
            f"and Aaron Joseph Berkhoff on Friday, November 20, and Saturday, November 21, 2026.\n"
            f"\n"
            f"Friday, November 20\n"
            f"\n"
            f"Wedding Rehearsal\n"
            f"6:30 PM (begins promptly; please arrive by 6:15 PM)\n"
            f"Corpus Christi Catholic Parish\n"
            f"2318 N Cascade Ave\n"
            f"Colorado Springs, CO 80907\n"
            f"\n"
            f"The rehearsal is for the wedding party and the parents of the bride and groom. "
            f"The rehearsal is expected to conclude at 7:00 PM.\n"
            f"\n"
            f"Rehearsal Dinner\n"
            f"7:30 PM\n"
            f"MacKenzie's Chop House\n"
            f"128 S Tejon St\n"
            f"Colorado Springs, CO 80903\n"
            f"\n"
            f"If you are not participating in the rehearsal, we look forward to welcoming you at the "
            f"rehearsal dinner beginning at 7:30 PM.\n"
            f"\n"
            f"Saturday, November 21\n"
            f"Nuptial Mass\n"
            f"1:30 PM (begins promptly)\n"
            f"Corpus Christi Catholic Parish\n"
            f"2318 N Cascade Ave\n"
            f"Colorado Springs, CO 80907\n"
            f"\n"
            f"The Nuptial Mass is expected to conclude around 2:45 PM.\n"
            f"\n"
            f"Dress Code\n"
            f"We kindly ask guests to dress in attire appropriate for a Catholic wedding.\n"
            f"\n"
            f"Women: Long dresses or skirts are encouraged with modest style choices. Shoulders may "
            f"be uncovered, but we ask that spaghetti straps or strapless dresses be paired with a "
            f"sweater, shawl, or similar cover.\n"
            f"Men: Dress shirt with a collar and tie; jacket is optional.\n"
            f"\n"
            f"Please visit the FAQ section of our wedding website for attire examples.\n"
            f"\n"
            f"Reception\n"
            f"4:45 PM\n"
            f"Red Rocks Barn\n"
            f"2700 Robinson St\n"
            f"Colorado Springs, CO 80904\n"
            f"\n"
            f"The reception will include a cocktail hour with appetizers from 4:45–6:00 PM, followed "
            f"by a buffet dinner and dancing.\n"
            f"\n"
            f"{rsvp_block}"
            f"\n"
            f"For the most up-to-date information, directions, FAQs, and other wedding details, "
            f"please visit our website:\n"
            f"\n"
            f"https://anna-aaron-wedding.fly.dev\n"
            f"\n"
            f"We are so grateful for your love and support and hope you'll be able to celebrate "
            f"with us!\n"
            f"\n"
            f"With love,\n"
            f"\n"
            f"Anna & Aaron"
        )

    return (
        f"Dear {guest_name},\n"
        f"\n"
        f"{urgency_note}"
        f"You are cordially invited to celebrate the wedding of Anna Pauline Hagen and Aaron Joseph "
        f"Berkhoff on Saturday, November 21, 2026.\n"
        f"\n"
        f"Nuptial Mass\n"
        f"1:30 PM (begins promptly)\n"
        f"Corpus Christi Catholic Parish\n"
        f"2318 N Cascade Ave\n"
        f"Colorado Springs, CO 80907\n"
        f"\n"
        f"The Nuptial Mass is expected to conclude around 2:45 PM.\n"
        f"\n"
        f"Dress Code\n"
        f"We kindly ask guests to dress in attire appropriate for a Catholic wedding.\n"
        f"\n"
        f"Women: Long dresses or skirts are encouraged with modest style choices. Shoulders may be "
        f"uncovered, but we ask that spaghetti straps or strapless dresses be paired with a sweater, "
        f"shawl, or similar cover.\n"
        f"Men: Dress shirt with a collar and slacks (no jeans); jacket and/or tie optional.\n"
        f"\n"
        f"Please visit the FAQ section of our wedding website for attire examples.\n"
        f"\n"
        f"Reception\n"
        f"4:45 PM\n"
        f"Red Rocks Barn\n"
        f"2700 Robinson St\n"
        f"Colorado Springs, CO 80904\n"
        f"\n"
        f"The reception will include a cocktail hour with appetizers from 4:45–6:00 PM, followed by "
        f"a buffet dinner and dancing.\n"
        f"\n"
        f"{rsvp_block}"
        f"\n"
        f"For the most up-to-date information, directions, FAQs, and other wedding details, please "
        f"visit our website:\n"
        f"\n"
        f"https://anna-aaron-wedding.fly.dev\n"
        f"\n"
        f"We are so grateful for your love and support and hope you'll be able to celebrate with us!\n"
        f"\n"
        f"With love,\n"
        f"\n"
        f"Anna & Aaron"
    )


_HTML_WRAPPER = """\
<html>
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
      @import url('https://fonts.googleapis.com/css2?family=Dancing+Script:wght@700\
&family=Playfair+Display:ital,wght@0,400;0,700;1,400&family=Inter:wght@400;500;600&display=swap');
    </style>
  </head>
  <body style="margin:0;padding:0;background:{champagne};">
    <table role="presentation" width="100%" cellpadding="0" cellspacing="0"
           style="background:{champagne};">
      <tr>
        <td align="center" style="padding:28px 12px;">
          <table role="presentation" width="600" cellpadding="0" cellspacing="0"
                 style="max-width:600px;width:100%;background:{cream};">
            <tr>
              <td style="background:{champagne};text-align:center;padding:44px 24px 28px;">
                <p style="margin:0 0 18px;font-family:{sans};font-size:11px;letter-spacing:4px;
                          text-transform:uppercase;color:{gold};">
                  November 21, 2026 &middot; Colorado Springs, CO
                </p>
                <p style="margin:0 0 20px;font-family:{script};font-size:54px;line-height:1;
                          color:{charcoal};">
                  Anna &amp; Aaron
                </p>
                <table role="presentation" align="center" cellpadding="0" cellspacing="0">
                  <tr>
                    <td style="width:60px;border-top:1px solid {gold};font-size:0;line-height:0;">&nbsp;</td>
                    <td style="padding:0 12px;color:{gold};font-size:13px;">&#10022;</td>
                    <td style="width:60px;border-top:1px solid {gold};font-size:0;line-height:0;">&nbsp;</td>
                  </tr>
                </table>
              </td>
            </tr>
            <tr>
              <td style="padding:36px 40px 8px;font-family:{sans};font-size:15px;
                         color:{charcoal};line-height:1.7;">
{content}
              </td>
            </tr>
            <tr>
              <td style="background:{charcoal};text-align:center;padding:34px 24px;">
                <p style="margin:0 0 8px;font-family:{script};font-size:26px;color:{ivory};">
                  Anna &amp; Aaron
                </p>
                <p style="margin:0;font-family:{sans};font-size:10px;letter-spacing:2px;
                          text-transform:uppercase;color:#c9c3b8;">
                  November 21, 2026 &middot; Corpus Christi Catholic Church &middot; Colorado Springs, CO
                </p>
              </td>
            </tr>
          </table>
        </td>
      </tr>
    </table>
  </body>
</html>
""".format(champagne=CHAMPAGNE, cream=CREAM, charcoal=CHARCOAL, gold=GOLD, ivory=IVORY,
           script=FONT_SCRIPT, sans=FONT_SANS, content="{content}")


def _label(text: str) -> str:
    return (
        f'<p style="margin:26px 0 6px;font-family:{FONT_SANS};font-size:11px;'
        f'letter-spacing:3px;text-transform:uppercase;color:{GOLD};font-weight:600;">'
        f"{html.escape(text)}</p>"
    )


def _event(time_text: str, venue_lines: list[str], note: str | None = None) -> str:
    venue_html = "<br>".join(html.escape(line) for line in venue_lines)
    note_html = (
        f'<p style="margin:10px 0 0;font-family:{FONT_SANS};font-size:14px;'
        f'color:{CHARCOAL_MUTED};line-height:1.6;">{note}</p>'
        if note else ""
    )
    return (
        f'<p style="margin:0 0 6px;font-family:{FONT_SERIF};font-size:21px;'
        f'font-weight:700;color:{CHARCOAL};">{html.escape(time_text)}</p>'
        f'<p style="margin:0;font-family:{FONT_SERIF};font-style:italic;font-size:15px;'
        f'color:{CHARCOAL_MUTED};line-height:1.5;">{venue_html}</p>'
        f"{note_html}"
    )


def _dress_code(men_line: str) -> str:
    return (
        f"{_label('Dress Code')}"
        f'<p style="margin:0 0 12px;font-family:{FONT_SANS};font-size:15px;'
        f'color:{CHARCOAL};line-height:1.7;">We kindly ask guests to dress in attire '
        f"appropriate for a Catholic wedding.</p>"
        f'<p style="margin:0;font-family:{FONT_SANS};font-size:15px;color:{CHARCOAL};'
        f'line-height:1.7;"><strong>Women:</strong> Long dresses or skirts are encouraged '
        f"with modest style choices. Shoulders may be uncovered, but we ask that spaghetti "
        f"straps or strapless dresses be paired with a sweater, shawl, or similar cover."
        f"<br><br><strong>Men:</strong> {men_line}</p>"
        f'<p style="margin:14px 0 0;font-family:{FONT_SANS};font-size:13px;'
        f'color:{CHARCOAL_MUTED};">Please visit the FAQ section of our wedding website '
        f"for attire examples.</p>"
    )


def _rsvp_block(guest_name: str, party: list[str], code: str, base_url: str,
                 last_call: bool = False) -> str:
    link = f"{base_url}/rsvp?code={code}"
    deadline_label = (
        "LAST CALL — Please RSVP by September 5, 2026" if last_call
        else "Please RSVP by September 4, 2026"
    )
    return (
        f'<div style="text-align:center;margin:32px 0 6px;">'
        f'<p style="margin:0 0 16px;font-family:{FONT_SANS};font-size:14px;color:{CHARCOAL};">'
        f"<strong>{deadline_label}</strong></p>"
        f'<a href="{link}" style="display:inline-block;background:{GOLD};color:{IVORY};'
        f'font-family:{FONT_SANS};font-size:12px;letter-spacing:2px;text-transform:uppercase;'
        f'text-decoration:none;padding:14px 42px;">RSVP Now</a>'
        f'<p style="margin:16px 0 0;font-family:{FONT_SANS};font-size:13px;'
        f'color:{CHARCOAL_MUTED};">Invite code: '
        f'<strong style="color:{CHARCOAL};">{code}</strong></p>'
        f"{_party_line_html(guest_name, party)}"
        f"</div>"
    )


def _closing() -> str:
    return (
        f'<p style="margin:28px 0 0;font-family:{FONT_SANS};font-size:15px;color:{CHARCOAL};'
        f'line-height:1.7;">For the most up-to-date information, directions, FAQs, and other '
        f'wedding details, please visit our website:<br>'
        f'<a href="https://anna-aaron-wedding.fly.dev" style="color:{GOLD};">'
        f"https://anna-aaron-wedding.fly.dev</a></p>"
        f'<p style="margin:20px 0 0;font-family:{FONT_SANS};font-size:15px;color:{CHARCOAL};'
        f'line-height:1.7;">We are so grateful for your love and support and hope you\'ll '
        f"be able to celebrate with us!</p>"
        f'<p style="margin:20px 0 0;font-family:{FONT_SERIF};font-size:16px;color:{CHARCOAL};">'
        f'With love,<br><span style="font-family:{FONT_SCRIPT};font-size:24px;">Anna &amp; Aaron'
        f"</span></p>"
    )


def _urgency_banner() -> str:
    return (
        f'<p style="margin:0 0 20px;padding:14px 18px;background:{CHAMPAGNE};'
        f'border-left:3px solid {GOLD};font-family:{FONT_SANS};font-size:14px;'
        f'color:{CHARCOAL};line-height:1.6;">This is a final reminder — we have not '
        f"yet received your RSVP, and the deadline is fast approaching.</p>"
    )


def build_body_html(guest_name: str, party: list[str], code: str, base_url: str,
                     rehearsal_invited: bool, last_call: bool = False) -> str:
    name_esc = html.escape(guest_name)
    rsvp_block = _rsvp_block(guest_name, party, code, base_url, last_call=last_call)
    urgency_banner = _urgency_banner() if last_call else ""

    if rehearsal_invited:
        content = (
            f'<p style="margin:0 0 20px;">Dear {name_esc},</p>'
            f"{urgency_banner}"
            f'<p style="margin:0 0 20px;">You are cordially invited to celebrate the '
            f"rehearsal and wedding of <strong>Anna Pauline Hagen</strong> and "
            f"<strong>Aaron Joseph Berkhoff</strong> on <strong>Friday, November 20</strong> "
            f"and <strong>Saturday, November 21, 2026</strong>.</p>"
            f"{_label('Friday, November 20 — Wedding Rehearsal')}"
            f'{_event("6:30 PM", ["Corpus Christi Catholic Parish", "2318 N Cascade Ave", "Colorado Springs, CO 80907"], "Begins promptly; please arrive by 6:15 PM. The rehearsal is for the wedding party and the parents of the bride and groom, and is expected to conclude at 7:00 PM.")}'
            f"{_label('Friday, November 20 — Rehearsal Dinner')}"
            f'{_event("7:30 PM", ["MacKenzie\'s Chop House", "128 S Tejon St", "Colorado Springs, CO 80903"], "If you are not participating in the rehearsal, we look forward to welcoming you here beginning at 7:30 PM.")}'
            f"{_label('Saturday, November 21 — Nuptial Mass')}"
            f'{_event("1:30 PM", ["Corpus Christi Catholic Parish", "2318 N Cascade Ave", "Colorado Springs, CO 80907"], "Begins promptly; expected to conclude around 2:45 PM.")}'
            f'{_dress_code("Dress shirt with a collar and tie; jacket is optional.")}'
            f"{_label('Reception')}"
            f'{_event("4:45 PM", ["Red Rocks Barn", "2700 Robinson St", "Colorado Springs, CO 80904"], "Cocktail hour with appetizers from 4:45–6:00 PM, followed by a buffet dinner and dancing.")}'
            f"{rsvp_block}"
            f"{_closing()}"
        )
    else:
        content = (
            f'<p style="margin:0 0 20px;">Dear {name_esc},</p>'
            f"{urgency_banner}"
            f'<p style="margin:0 0 20px;">You are cordially invited to celebrate the '
            f"wedding of <strong>Anna Pauline Hagen</strong> and <strong>Aaron Joseph "
            f"Berkhoff</strong> on <strong>Saturday, November 21, 2026</strong>.</p>"
            f"{_label('Nuptial Mass')}"
            f'{_event("1:30 PM", ["Corpus Christi Catholic Parish", "2318 N Cascade Ave", "Colorado Springs, CO 80907"], "Begins promptly; expected to conclude around 2:45 PM.")}'
            f'{_dress_code("Dress shirt with a collar and slacks (no jeans); jacket and/or tie optional.")}'
            f"{_label('Reception')}"
            f'{_event("4:45 PM", ["Red Rocks Barn", "2700 Robinson St", "Colorado Springs, CO 80904"], "Cocktail hour with appetizers from 4:45–6:00 PM, followed by a buffet dinner and dancing.")}'
            f"{rsvp_block}"
            f"{_closing()}"
        )

    return _HTML_WRAPPER.format(content=content)


def send_email(smtp_cfg: dict, to_email: str, subject: str, text_body: str, html_body: str):
    msg = MIMEMultipart("alternative")
    msg["Subject"] = subject
    msg["From"] = smtp_cfg["from"]
    msg["To"] = to_email
    msg.attach(MIMEText(text_body, "plain"))
    msg.attach(MIMEText(html_body, "html"))

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

    guests = load_guests(args.db, resend=args.resend, test_only=args.test, last_call=args.last_call)

    if not guests:
        if args.last_call:
            print("No guests to email — everyone has already RSVP'd.")
        else:
            suffix = " (use --resend to re-send)" if not args.resend else ""
            print(f"No guests to email{suffix}.")
        return

    mode = "[DRY RUN] " if args.dry_run else ""
    if args.last_call:
        mode += "[LAST CALL] "
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

        text_body = build_body_text(g["name"], g["party"], g["invite_code"], args.base_url,
                                     g["rehearsal_invited"], last_call=args.last_call)
        html_body = build_body_html(g["name"], g["party"], g["invite_code"], args.base_url,
                                     g["rehearsal_invited"], last_call=args.last_call)
        subject = (
            "Last Call: Please RSVP by September 5 — Anna & Aaron's Wedding" if args.last_call
            else "You're invited — Anna & Aaron's Wedding"
        )

        try:
            send_email(smtp_cfg, recipient, subject, text_body, html_body)
            if not args.test_to and not args.last_call:
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
