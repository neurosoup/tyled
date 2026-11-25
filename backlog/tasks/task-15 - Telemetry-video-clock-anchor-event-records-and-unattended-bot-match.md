---
id: TASK-15
title: 'Telemetry: video clock anchor, event records and unattended bot match'
status: In Progress
assignee: []
created_date: '2026-11-24 15:00'
updated_date: '2026-11-24 15:00'
labels:
  - telemetry
dependencies: []
ordinal: 4000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make play_trace.jsonl usable to find and cut clips from an OBS recording of a bot-vs-bot session. Adds a clock anchor (unix_ms, rt), event records, and a launch mode for N unattended bot-vs-bot rounds. A devlog tool in another repo only reads the trace.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 session record has schema_version, unix_ms, t_at_write, rt_at_write and map
- [ ] #2 Every record has t and rt; outcome has t, rt and round
- [ ] #3 Event records: round_start, claim, hit, knockback, parry, ability_used, charges_empty, session_end
- [ ] #4 --bot-vs-bot, --rounds N, --after-last-round stay|exit and --loadout-p1/p2 start an unattended match
- [ ] #5 doc-21 has the full schema and one example line per kind
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented and unit tested (36 tests pass). Not yet verified in a real unattended run: run with --bot-vs-bot --rounds N --after-last-round exit and check play_trace.jsonl.
<!-- SECTION:NOTES:END -->
