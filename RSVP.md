# RSVP Plan

## Requirements
- look up rsvp by name
- able to rsvp for other in their party (plus ones and family units)
- RSVP information shall be provide in a sql database
- allow guess to provide people they know for to be used by seating chart (Who do you know who is attending, this will be used when deciding seating chart)
- need an option to RSVP for rehearsal dinner and wedding reception. Some guests will be invited to the rehearsal dinner and the reception. 
- If a guest is not invited to rehearsal dinner do not give them an option to rsvp
- Need a python script that will read from an excel spreadsheet of the rsvp where the columns are name, email, invite to rehearsal (true or false), associate invites for families and plus ones (dynamic in size so maybe use a column key word naming convention. the additional columns will be names of the associated guests)
- option to provide invite code or link per guest. Randomly excel spreadsheet will have the 4 digit code.
- drop down menu of guests to rsvp if no code is given.
- thinking about a backend to provide a link to use specific invite and an automated email system to send out RSVP notifications when spreadsheet is converted to SQL.
- Need a code random test guest number with mock information for testing
- Need a way to make clear the database (and verify if the developer is sure they wish to purge the data with two checks (Are you sure, Are you really sure?))
- Open to other additions and suggestions
  

python3 scripts/import_guests.py data/wedding_invites.csv --db wedding.db   # updates emails, preserves invite_sent
python3 scripts/send_invites.py --db wedding.db --dry-run                   # verify: only the new people listed
python3 scripts/send_invites.py --db wedding.db                             # sends to them only


set -a; source .env; set +a
python3 scripts/send_invites.py --db wedding.db --dry-run   # final check
python3 scripts/send_invites.py --db wedding.db             # send to all 39

## Updated Wedding only email

Dear Family and Friends,

You are cordially invited to celebrate the wedding of Anna Pauline Hagen and Aaron Joseph Berkhoff on Saturday, November 21, 2026.

**Nuptial Mass**
**1:30 PM** (begins promptly)
Corpus Christi Catholic Parish
2318 N Cascade Ave
Colorado Springs, CO 80907

The Nuptial Mass is expected to conclude around 2:45 PM.

**Dress Code**
We kindly ask guests to dress in attire appropriate for a Catholic wedding.

**Women**: Long dresses or skirts are encouraged with modest style choices. Shoulders may be uncovered, but we ask that spaghetti straps or strapless dresses be paired with a sweater, shawl, or similar cover.
**Men**: Dress shirt with a collar and slacks (no jeans); jacket and/or tie optional.

Please visit the FAQ section of our wedding website for attire examples.

**Reception**
**4:45 PM**
Red Rocks Barn
2700 Robinson St
Colorado Springs, CO 80904

The reception will include a cocktail hour with appetizers from **4:45–6:00 PM**, followed by a buffet dinner and dancing.

**Please RSVP by September 4, 2026.**

For the most up-to-date information, directions, FAQs, and other wedding details, please visit our website:

https://anna-aaron-wedding.fly.dev

We are so grateful for your love and support and hope you'll be able to celebrate with us!

With love,

Anna & Aaron

## Updated Wedding and Rehearsal Dinner invite

Dear Family and Friends,

You are cordially invited to celebrate the rehearsal and wedding of Anna Pauline Hagen and Aaron Joseph Berkhoff on Friday, November 20, and Saturday, November 21, 2026.

Friday, November 20

Wedding Rehearsal
6:30 PM (begins promptly; please arrive by 6:15 PM)
Corpus Christi Catholic Parish
2318 N Cascade Ave
Colorado Springs, CO 80907

The rehearsal is for the wedding party and the parents of the bride and groom. The rehearsal is expected to conclude at 7:00 PM.

Rehearsal Dinner
7:30 PM
MacKenzie's Chop House
128 S Tejon St
Colorado Springs, CO 80903

If you are not participating in the rehearsal, we look forward to welcoming you at the rehearsal dinner beginning at 7:30 PM.

Saturday, November 21
Nuptial Mass
1:30 PM (begins promptly)
Corpus Christi Catholic Parish
2318 N Cascade Ave
Colorado Springs, CO 80907

The Nuptial Mass is expected to conclude around 2:45 PM.

Dress Code
We kindly ask guests to dress in attire appropriate for a Catholic wedding.

*Women: Long dresses or skirts are encouraged with modest style choices. Shoulders may be uncovered, but we ask that spaghetti straps or strapless dresses be paired with a sweater, shawl, or similar cover.
*Men: Dress shirt with a collar and tie; jacket is optional.

Please visit the FAQ section of our wedding website for attire examples.

Reception
4:45 PM
Red Rocks Barn
2700 Robinson St
Colorado Springs, CO 80904

The reception will include a cocktail hour with appetizers from 4:45–6:00 PM, followed by a buffet dinner and dancing.

Please RSVP by September 4, 2026.

For the most up-to-date information, directions, FAQs, and other wedding details, please visit our website:

https://anna-aaron-wedding.fly.dev

We are so grateful for your love and support and hope you'll be able to celebrate with us!

With love,

Anna & Aaron
