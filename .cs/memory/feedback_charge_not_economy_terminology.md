---
name: feedback-charge-not-economy-terminology
description: "In Tyled, name charge/BeamCharges-related plugins, files, and config sections \"charge\", not \"economy\""
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 0dc473fc-54c7-46aa-9de2-8bb99ad4272e
  modified: 2026-08-13T08:34:04.074Z
---

Use "charge" terminology, not "economy", when naming plugins/files/config
sections that deal with `BeamCharges` (regen, refund, cost policy) in Tyled.

**Why:** DECKBUILDING.md §6 originally floated "a future charge/economy
plugin" for this domain. When asked to plan extracting it, the user
explicitly rejected the "economy" half as less clear than "charge" — the
plugin's job is to own the `BeamCharges` resource, so name it after that.

**How to apply:** When implementing anything from DECKBUILDING.md's
charge-economy family (Solar Panels, Salvage, Tithe, Frugal Frontier,
Battery Cap, Capacitor, etc. — §3 "Charge-economy modifiers"), use `charge`
in code identifiers (`src/plugins/charge.rs`, `ChargeConfig`,
`plugins::charge::plugin`) even where DECKBUILDING.md's prose still says
"economy". If DECKBUILDING.md's own prose gets touched during that work,
prefer updating it to say "charge" too rather than leaving the two
terminologies split across doc and code.
