---
description: One agent tick — read inbox, pick one ticket, drive it to terminal, exit.
---

# /agent-tick

You are running one iteration of your agent loop. Your persona, project,
and protocol are described in `CLAUDE.md`.

## 1. Stop check

If `.claude/swarm-stop` exists, exit cleanly without doing any work.

## 2. Inbox

Call `wn_inbox(limit=20)`. New comments mentioning you (`@{{persona_slug}}`),
`[state-update]` notes, and replies on tickets you own are fresh context.

## 3. Pick work

Call `wn_list_my_tickets()`.

- If any ticket is **InProgress** and assigned to you → resume the
  highest-priority one.
- Else if any ticket is **Open** and assigned to you →
  `wn_claim_ticket(id)`. On HTTP 412 (someone else just took it),
  re-list and try again.
- Else if you are the tech-lead and any **Open Epic** is assigned to you
  with no children → decompose it (see §5 below) and exit.
- Else → exit cleanly (queue empty).

## 4. Plan

Post a 3–6 bullet plan as your first comment on the ticket via
`wn_comment`. This is durable: a fresh session resuming the ticket later
uses your plan + subsequent progress comments to catch up.

## 5. Decompose (tech-lead only)

For each Epic ticket assigned to you with no children:

1. Read its full description and any prior comments.
2. Pick 2–5 subtasks with the most relevant assignee personas. **Choose
   `assignee_persona` from the "Peers in this project" section of
   `CLAUDE.md`** — that is the authoritative roster of who is actually
   deployed here. Never invent a slug. If nothing matches, leave the
   subtask unassigned rather than misrouting it.
3. For each subtask, call:
   ```
   wn_create_subtask(
     parent_id=<epic_id>,
     title="<short title>",
     assignee_persona="<slug>",
     priority="<Low|Medium|High|Critical>",
     description="<acceptance criteria>",
     ticket_type="Task"
   )
   ```
4. `wn_comment(epic_id, "decomposed into <N> subtasks: WN-X (frontend), …")`
5. Set the Epic's status to `InProgress` via `wn_set_status` (if your
   capabilities include it) or by closing the comment with `[status:
   InProgress]` for the next tick.

## 6. Execute (worker personas)

Drive the chosen ticket to a terminal state:

- **Done**: `wn_finish(id, summary)` — moves to `Review`.
- **Blocked / need help**: `wn_handoff(id, to_persona, status="Open", note=...)`
  with a precise question.
- **Stable Blocked**: leave InProgress with a comment describing what
  input you need and from whom.

## 7. Exit

Do NOT call `ScheduleWakeup` — this is cron mode; the OS fires the next
tick on schedule. The advisory `flock` and the database tick lock
guarantee no overlapping ticks for this deployment.
