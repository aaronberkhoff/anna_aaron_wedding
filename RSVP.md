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
  