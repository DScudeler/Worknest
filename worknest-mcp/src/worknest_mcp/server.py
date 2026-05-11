"""MCP server exposing Worknest REST as A2A tools for swarm agents.

Configuration (env, set by swarm CLI in settings.json):
  WORKNEST_URL          base URL (e.g. http://localhost:3000)
  WORKNEST_PROJECT_ID   the swarm project UUID
  WORKNEST_PERSONA      this agent's persona name
  WORKNEST_TOKEN_FILE   path to file containing the JWT
  WORKNEST_PERSONAS     path to {persona: user_id} JSON map
                        (default: <token_file>/../../personas.json)
  WORKNEST_INBOX_STATE  path to last-seen-comment-ts file
                        (default: <token_file>/../inbox.json)
  WORKNEST_STATE_TICKET id of the [STATE] swarm-config ticket; agents
                        read it via wn_get_state() and tech-lead writes
                        it via wn_set_state(). Optional — if missing,
                        the tool falls back to a magic-title lookup.
"""

from __future__ import annotations

import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import httpx
from mcp.server.fastmcp import FastMCP


PRIORITY_ORDER = {"critical": 0, "high": 1, "medium": 2, "low": 3}
TICKET_TYPES = {"task", "bug", "feature", "epic"}
STATUSES = {"open", "inprogress", "review", "done", "closed"}


def _env(name: str, required: bool = True, default: str | None = None) -> str:
    val = os.environ.get(name, default)
    if required and not val:
        print(f"worknest-mcp: missing env {name}", file=sys.stderr)
        sys.exit(2)
    return val or ""


URL = _env("WORKNEST_URL").rstrip("/")
PROJECT_ID = _env("WORKNEST_PROJECT_ID")
PERSONA = _env("WORKNEST_PERSONA")
TOKEN_FILE = _env("WORKNEST_TOKEN_FILE")
PERSONAS_FILE = _env(
    "WORKNEST_PERSONAS",
    required=False,
    default=str(Path(TOKEN_FILE).parent.parent / "personas.json"),
)
INBOX_STATE = _env(
    "WORKNEST_INBOX_STATE",
    required=False,
    default=str(Path(TOKEN_FILE).parent / "inbox.json"),
)
STATE_TICKET_ENV = _env("WORKNEST_STATE_TICKET", required=False, default="")
STATE_TITLE = "[STATE] swarm config"


def _token() -> str:
    return Path(TOKEN_FILE).read_text().strip()


def _personas() -> dict[str, str]:
    p = Path(PERSONAS_FILE)
    if not p.exists():
        return {}
    return json.loads(p.read_text())


def _persona_to_user(persona: str) -> str | None:
    return _personas().get(persona)


def _user_to_persona(user_id: str) -> str | None:
    for persona, uid in _personas().items():
        if uid == user_id:
            return persona
    return None


def _client() -> httpx.Client:
    return httpx.Client(
        base_url=URL,
        headers={"Authorization": f"Bearer {_token()}"},
        timeout=30.0,
    )


def _err(resp: httpx.Response, action: str) -> dict[str, Any]:
    try:
        body = resp.json()
    except Exception:
        body = {"error": resp.text[:200]}
    return {
        "ok": False,
        "action": action,
        "status_code": resp.status_code,
        "error": body.get("error") or body,
    }


mcp = FastMCP("worknest")


# ──────────────────────────────────────────────────────────────────────
# Read-side
# ──────────────────────────────────────────────────────────────────────

@mcp.tool()
def wn_list_my_tickets() -> dict[str, Any]:
    """List tickets assigned to this persona that are not Done/Closed,
    sorted by priority (Critical→Low) then updated_at desc."""
    me = _persona_to_user(PERSONA)
    if not me:
        return {"ok": False, "error": f"persona {PERSONA!r} not in personas.json"}
    with _client() as c:
        r = c.get("/api/tickets")
    if r.status_code != 200:
        return _err(r, "list_tickets")
    tickets = [
        t for t in r.json()
        if t.get("project_id") == PROJECT_ID
        and t.get("assignee_id") == me
        and t.get("status", "").lower() not in {"done", "closed"}
    ]
    tickets.sort(key=lambda t: (
        PRIORITY_ORDER.get(t.get("priority", "medium").lower(), 9),
        -datetime.fromisoformat(t["updated_at"].replace("Z", "+00:00")).timestamp(),
    ))
    return {"ok": True, "count": len(tickets), "tickets": tickets}


@mcp.tool()
def wn_get_ticket(ticket_id: str) -> dict[str, Any]:
    """Fetch a ticket and its full comment thread."""
    with _client() as c:
        rt = c.get(f"/api/tickets/{ticket_id}")
        if rt.status_code != 200:
            return _err(rt, "get_ticket")
        rc = c.get(f"/api/tickets/{ticket_id}/comments")
        if rc.status_code != 200:
            return _err(rc, "get_comments")
    return {"ok": True, "ticket": rt.json(), "comments": rc.json()}


@mcp.tool()
def wn_inbox(limit: int = 50) -> dict[str, Any]:
    """Comments on tickets you're assigned to, posted since your last
    inbox check. Updates the high-water mark on success."""
    me = _persona_to_user(PERSONA)
    if not me:
        return {"ok": False, "error": f"persona {PERSONA!r} not in personas.json"}
    p = Path(INBOX_STATE)
    last_ts = "1970-01-01T00:00:00+00:00"
    if p.exists():
        last_ts = json.loads(p.read_text()).get("last_ts", last_ts)
    last_dt = datetime.fromisoformat(last_ts)

    new_items: list[dict[str, Any]] = []
    with _client() as c:
        rt = c.get("/api/tickets")
        if rt.status_code != 200:
            return _err(rt, "list_tickets")
        my_tickets = [
            t for t in rt.json()
            if t.get("project_id") == PROJECT_ID
            and t.get("assignee_id") == me
        ]
        for t in my_tickets:
            rc = c.get(f"/api/tickets/{t['id']}/comments")
            if rc.status_code != 200:
                continue
            for cm in rc.json():
                ct = datetime.fromisoformat(cm["created_at"].replace("Z", "+00:00"))
                if ct > last_dt and cm.get("user_id") != me:
                    new_items.append({
                        "ticket_id": t["id"],
                        "ticket_title": t.get("title"),
                        "from_persona": _user_to_persona(cm.get("user_id", "")) or cm.get("user_id"),
                        "body": cm.get("content"),
                        "at": cm.get("created_at"),
                    })
    new_items.sort(key=lambda x: x["at"])
    new_items = new_items[-limit:]

    if new_items:
        newest = max(item["at"] for item in new_items)
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(json.dumps({"last_ts": newest}))
    return {"ok": True, "count": len(new_items), "messages": new_items}


# ──────────────────────────────────────────────────────────────────────
# Write-side
# ──────────────────────────────────────────────────────────────────────

@mcp.tool()
def wn_claim_ticket(ticket_id: str) -> dict[str, Any]:
    """Atomically move an Open ticket to InProgress using If-Match.
    Returns 412 details if another agent claimed it first."""
    with _client() as c:
        r = c.get(f"/api/tickets/{ticket_id}")
        if r.status_code != 200:
            return _err(r, "claim:get")
        ticket = r.json()
        if ticket.get("status", "").lower() != "open":
            return {"ok": False, "error": f"ticket is {ticket['status']}, not Open"}
        etag = ticket["updated_at"]
        r2 = c.put(
            f"/api/tickets/{ticket_id}",
            headers={"If-Match": etag},
            json={"status": "inprogress"},
        )
    if r2.status_code == 412:
        return {"ok": False, "error": "race lost — ticket modified by another agent",
                "status_code": 412}
    if r2.status_code != 200:
        return _err(r2, "claim:put")
    return {"ok": True, "ticket": r2.json()}


@mcp.tool()
def wn_comment(ticket_id: str, body: str) -> dict[str, Any]:
    """Post a comment on a ticket — the A2A message primitive."""
    prefix = f"[{PERSONA}] "
    payload = {"content": body if body.startswith(prefix) else prefix + body}
    with _client() as c:
        r = c.post(f"/api/tickets/{ticket_id}/comments", json=payload)
    if r.status_code != 200:
        return _err(r, "comment")
    return {"ok": True, "comment": r.json()}


@mcp.tool()
def wn_handoff(
    ticket_id: str,
    to_persona: str,
    status: str = "open",
    note: str = "",
) -> dict[str, Any]:
    """Reassign to another persona and optionally change status. Posts a
    handoff comment recording the reason."""
    target_uid = _persona_to_user(to_persona)
    if not target_uid:
        return {"ok": False, "error": f"unknown persona {to_persona!r}"}
    if status.lower() not in STATUSES:
        return {"ok": False, "error": f"bad status {status!r}"}

    note_body = (
        f"[handoff] {PERSONA} → {to_persona}: {note}" if note
        else f"[handoff] {PERSONA} → {to_persona}"
    )
    with _client() as c:
        rc = c.post(f"/api/tickets/{ticket_id}/comments", json={"content": note_body})
        if rc.status_code != 200:
            return _err(rc, "handoff:comment")
        rg = c.get(f"/api/tickets/{ticket_id}")
        if rg.status_code != 200:
            return _err(rg, "handoff:get")
        etag = rg.json()["updated_at"]
        r = c.put(
            f"/api/tickets/{ticket_id}",
            headers={"If-Match": etag},
            json={"assignee_id": target_uid, "status": status.lower()},
        )
    if r.status_code != 200:
        return _err(r, "handoff:put")
    return {"ok": True, "ticket": r.json()}


@mcp.tool()
def wn_finish(
    ticket_id: str,
    summary: str,
    commit_shas: list[str] | None = None,
    target_status: str = "review",
) -> dict[str, Any]:
    """Mark your work done. Workers default to status=Review (tech-lead
    will merge and close). Tech-lead passes target_status='done' after merge."""
    if target_status.lower() not in {"review", "done"}:
        return {"ok": False, "error": "target_status must be review or done"}

    sha_line = ""
    if commit_shas:
        sha_line = "\n\nCommits: " + ", ".join(f"`{s[:12]}`" for s in commit_shas)
    body = f"[finish] {summary}{sha_line}"

    update: dict[str, Any] = {"status": target_status.lower()}
    if target_status.lower() == "review":
        tech_lead_uid = _persona_to_user("tech-lead")
        if tech_lead_uid:
            update["assignee_id"] = tech_lead_uid

    with _client() as c:
        rc = c.post(f"/api/tickets/{ticket_id}/comments", json={"content": body})
        if rc.status_code != 200:
            return _err(rc, "finish:comment")
        rg = c.get(f"/api/tickets/{ticket_id}")
        if rg.status_code != 200:
            return _err(rg, "finish:get")
        etag = rg.json()["updated_at"]
        r = c.put(
            f"/api/tickets/{ticket_id}",
            headers={"If-Match": etag},
            json=update,
        )
    if r.status_code != 200:
        return _err(r, "finish:put")
    return {"ok": True, "ticket": r.json()}


@mcp.tool()
def wn_create_ticket(
    title: str,
    description: str = "",
    ticket_type: str = "bug",
    priority: str = "medium",
    assignee_persona: str | None = None,
) -> dict[str, Any]:
    """Create a new top-level ticket (no parent). If assignee_persona is
    given, assigns the ticket to that persona's identity user; otherwise
    leaves it unassigned for triage."""
    if ticket_type.lower() not in TICKET_TYPES:
        return {"ok": False, "error": f"bad ticket_type {ticket_type!r}"}
    if priority.lower() not in PRIORITY_ORDER:
        return {"ok": False, "error": f"bad priority {priority!r}"}
    target_uid: str | None = None
    if assignee_persona:
        target_uid = _persona_to_user(assignee_persona)
        if not target_uid:
            return {"ok": False, "error": f"unknown persona {assignee_persona!r}"}

    body_lines = [description] if description else []
    body_lines.append(f"\nFiled by: {PERSONA}")
    payload = {
        "project_id": PROJECT_ID,
        "title": title,
        "description": "\n".join(body_lines),
        "ticket_type": ticket_type.lower(),
        "priority": priority.lower(),
    }
    with _client() as c:
        r = c.post("/api/tickets", json=payload)
        if r.status_code != 200:
            return _err(r, "ticket:create")
        new_ticket = r.json()
        if target_uid:
            ra = c.put(
                f"/api/tickets/{new_ticket['id']}",
                headers={"If-Match": new_ticket["updated_at"]},
                json={"assignee_id": target_uid},
            )
            if ra.status_code != 200:
                return _err(ra, "ticket:assign")
            new_ticket = ra.json()
    return {"ok": True, "ticket": new_ticket}


@mcp.tool()
def wn_create_subtask(
    parent_id: str,
    title: str,
    assignee_persona: str,
    priority: str = "medium",
    description: str = "",
    ticket_type: str = "task",
) -> dict[str, Any]:
    """Create a child ticket assigned to a persona. Comments on the parent
    linking the new child."""
    if ticket_type.lower() not in TICKET_TYPES:
        return {"ok": False, "error": f"bad ticket_type {ticket_type!r}"}
    if priority.lower() not in PRIORITY_ORDER:
        return {"ok": False, "error": f"bad priority {priority!r}"}
    target_uid = _persona_to_user(assignee_persona)
    if not target_uid:
        return {"ok": False, "error": f"unknown persona {assignee_persona!r}"}

    body_lines = [description] if description else []
    body_lines.append(f"\nParent: WN-{parent_id}")
    body_lines.append(f"Created by: {PERSONA}")
    payload = {
        "project_id": PROJECT_ID,
        "title": title,
        "description": "\n".join(body_lines),
        "ticket_type": ticket_type.lower(),
        "priority": priority.lower(),
    }
    with _client() as c:
        r = c.post("/api/tickets", json=payload)
        if r.status_code != 200:
            return _err(r, "subtask:create")
        new_ticket = r.json()
        ra = c.put(
            f"/api/tickets/{new_ticket['id']}",
            headers={"If-Match": new_ticket["updated_at"]},
            json={"assignee_id": target_uid},
        )
        if ra.status_code != 200:
            return _err(ra, "subtask:assign")
        c.post(
            f"/api/tickets/{parent_id}/comments",
            json={"content": f"[subtask] created WN-{new_ticket['id']} → @{assignee_persona}: {title}"},
        )
    return {"ok": True, "ticket": ra.json()}


# ──────────────────────────────────────────────────────────────────────
# Project state — the live, mutable spec every persona reads each tick
# ──────────────────────────────────────────────────────────────────────

def _resolve_state_ticket_id(client: httpx.Client) -> str | None:
    """Return the state ticket id. Prefer env, else look up by magic title."""
    if STATE_TICKET_ENV:
        return STATE_TICKET_ENV
    r = client.get("/api/tickets")
    if r.status_code != 200:
        return None
    for t in r.json():
        if t.get("project_id") == PROJECT_ID and t.get("title") == STATE_TITLE:
            return t["id"]
    return None


@mcp.tool()
def wn_get_state() -> dict[str, Any]:
    """Read the live project state. Every persona should call this at the
    start of every tick — values like test_cmd, build_cmd, conventions,
    or freeze can change between ticks.

    Returns: {"ok": bool, "state": dict, "etag": str, "ticket_id": str}
    """
    with _client() as c:
        tid = _resolve_state_ticket_id(c)
        if not tid:
            return {"ok": False, "error": "state ticket not found",
                    "state": {}, "etag": None}
        r = c.get(f"/api/tickets/{tid}")
        if r.status_code != 200:
            return _err(r, "state:get")
    ticket = r.json()
    desc = ticket.get("description") or "{}"
    try:
        state = json.loads(desc)
    except json.JSONDecodeError as e:
        return {"ok": False, "error": f"state ticket description is not JSON: {e}",
                "raw": desc, "etag": ticket["updated_at"], "ticket_id": tid}
    return {"ok": True, "state": state, "etag": ticket["updated_at"],
            "ticket_id": tid}


@mcp.tool()
def wn_set_state(patch: dict[str, Any], reason: str) -> dict[str, Any]:
    """Merge `patch` into the live project state. Tech-lead only — other
    personas should propose changes via comment instead.

    Posts a `[state-update]` comment recording the diff and reason so the
    full history is auditable on the state ticket. Uses If-Match for
    race-free updates; on 412 the caller should retry after re-reading.
    """
    if PERSONA != "tech-lead":
        return {"ok": False, "error": (
            "wn_set_state is tech-lead only. Other personas: comment on "
            "the state ticket asking tech-lead to make the change."
        )}
    if not isinstance(patch, dict) or not patch:
        return {"ok": False, "error": "patch must be a non-empty dict"}
    if not reason or not reason.strip():
        return {"ok": False, "error": "reason is required (audit trail)"}

    with _client() as c:
        tid = _resolve_state_ticket_id(c)
        if not tid:
            return {"ok": False, "error": "state ticket not found"}
        r = c.get(f"/api/tickets/{tid}")
        if r.status_code != 200:
            return _err(r, "state:get")
        ticket = r.json()
        try:
            current = json.loads(ticket.get("description") or "{}")
        except json.JSONDecodeError:
            current = {}
        diff = {k: {"old": current.get(k), "new": v}
                for k, v in patch.items() if current.get(k) != v}
        if not diff:
            return {"ok": True, "state": current, "noop": True,
                    "message": "patch matches current state"}
        merged = {**current, **patch}
        new_desc = json.dumps(merged, indent=2, sort_keys=True)
        etag = ticket["updated_at"]
        # PUT description + If-Match
        r2 = c.put(
            f"/api/tickets/{tid}",
            headers={"If-Match": etag},
            json={"description": new_desc},
        )
        if r2.status_code == 412:
            return {"ok": False, "error": "state ticket changed concurrently — re-read and retry",
                    "status_code": 412}
        if r2.status_code != 200:
            return _err(r2, "state:put")
        # audit comment
        diff_md = "\n".join(
            f"- `{k}`: `{json.dumps(d['old'])}` → `{json.dumps(d['new'])}`"
            for k, d in diff.items()
        )
        c.post(
            f"/api/tickets/{tid}/comments",
            json={"content": f"[state-update] {reason}\n\n{diff_md}"},
        )
    return {"ok": True, "state": merged, "diff": diff}


@mcp.tool()
def wn_attach_text(ticket_id: str, filename: str, body: str) -> dict[str, Any]:
    """Attach a text blob (test logs, diffs) as a file on the ticket."""
    files = {"file": (filename, body.encode("utf-8"), "text/plain")}
    with _client() as c:
        # httpx multipart uploads use `files=`; bearer header is on the client
        r = c.post(f"/api/tickets/{ticket_id}/attachments", files=files)
    if r.status_code != 200:
        return _err(r, "attach")
    return {"ok": True, "attachment": r.json()}


# ──────────────────────────────────────────────────────────────────────
# Resources — for diagnostic reads from the host
# ──────────────────────────────────────────────────────────────────────

@mcp.resource("worknest://me")
def me() -> str:
    """My identity inside the swarm."""
    return json.dumps({
        "persona": PERSONA,
        "user_id": _persona_to_user(PERSONA),
        "project_id": PROJECT_ID,
        "url": URL,
        "personas_known": list(_personas().keys()),
    }, indent=2)


def main() -> None:
    mcp.run()


if __name__ == "__main__":
    main()
