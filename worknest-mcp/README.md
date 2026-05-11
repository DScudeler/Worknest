# worknest-mcp

MCP server wrapping the Worknest REST API as A2A tools for autonomous agents.

This package is consumed by Worknest's agent activation pipeline (see
`crates/worknest-api/src/agents/`). When a deployment is activated, the
rendered `.mcp.json` launches the server via
`uv run --directory <this-dir> worknest-mcp`.

## Install

```bash
cd worknest-mcp
uv sync
```

Run this once after cloning Worknest. Subsequent agent ticks reuse the
populated `.venv`. If `.venv` is missing, the first tick triggers `uv sync`
implicitly — but it's faster to do it up front.

## Configuration (env vars)

The server is launched per-tick by Claude Code with these vars set from
Worknest's rendered `.mcp.json`. Operators do not set them by hand.

| Var | Required | Default | Purpose |
|---|---|---|---|
| `WORKNEST_URL` | ✓ | — | base URL, e.g. `http://localhost:3000` |
| `WORKNEST_PROJECT_ID` | ✓ | — | UUID of the project the agent serves |
| `WORKNEST_PERSONA` | ✓ | — | this agent's persona slug |
| `WORKNEST_TOKEN_FILE` | ✓ | — | path to file with the JWT |
| `WORKNEST_PERSONAS` | | `<token_file>/../../personas.json` | persona→user_id map |
| `WORKNEST_INBOX_STATE` | | `<token_file>/../inbox.json` | last-seen comment ts |

## Tools

- `wn_list_my_tickets()` – my tickets, prioritised
- `wn_get_ticket(id)` – ticket + comments
- `wn_inbox(limit)` – new comments since last check
- `wn_claim_ticket(id)` – race-free Open → InProgress (uses `If-Match`)
- `wn_comment(id, body)` – agent-to-agent message
- `wn_handoff(id, to_persona, status, note)` – reassign
- `wn_finish(id, summary, commit_shas?, target_status?)` – Review (worker)
  or Done (tech-lead)
- `wn_create_subtask(parent_id, title, assignee_persona, priority, description, ticket_type)`
- `wn_attach_text(id, filename, body)` – text attachment for diffs/logs
