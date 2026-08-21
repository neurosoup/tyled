---
name: No Co-Authored-By in commits
description: User does not want Co-Authored-By lines appended to commit messages
type: feedback
originSessionId: 45a33a89-719d-481f-b67f-117269fc6f22
---
Do not add "Co-Authored-By: Claude ..." lines to git commit messages.

**Why:** User explicitly rejected a commit that included the Co-Authored-By attribution line.

**How to apply:** When creating any git commit, omit the Co-Authored-By trailer entirely.
