"""
set_up_server.py
================
Full setup + verification script for the Granda Debate Tournament Planner.
Endpoints verified against /openapi.json
 
Valid roles (from API spec Role enum): Organizer, Judge, Marshal
 
Usage:
  pip install requests
  python scripts/set_up_server.py
"""
 
import requests
import secrets
import string
import json
import sys
 
# ─────────────────────────────────────────────
#  CONFIG
# ─────────────────────────────────────────────
 
BASE_URL   = "https://granda-backend.debateco.re"              # switch to https://granda-backend.debateco.re for prod
ADMIN_USER = "admin"
ADMIN_PASS = "even&headless&fastball&wager&reprise&unblessed&scorebook"                              # local default; prod uses the long passphrase
 
ORGANIZATIONS = ["DebateLab", "ZSK", "Buster"]
 
# Valid roles per API spec: Organizer | Judge | Marshal
# "Tabmaster" does NOT exist in the API — replaced with "Organizer"
ROLES         = ["Organizer", "Judge", "Marshal"]
TOURNAMENTS_N = 3
 
# ─────────────────────────────────────────────
#  CONSOLE COLOURS
# ─────────────────────────────────────────────
 
BOLD   = "\033[1m"
GREEN  = "\033[92m"
RED    = "\033[91m"
CYAN   = "\033[96m"
YELLOW = "\033[93m"
RESET  = "\033[0m"
 
def ok(msg):   print(f"  {GREEN}✓{RESET} {msg}")
def err(msg):  print(f"  {RED}✗{RESET} {msg}")
def warn(msg): print(f"  {YELLOW}!{RESET} {msg}")
def hdr(msg):  print(f"\n{BOLD}{CYAN}{'─'*55}\n  {msg}\n{'─'*55}{RESET}")
 
def base_url(path: str) -> str:
    return BASE_URL.rstrip("/") + path
 
def generate_password(length: int = 20) -> str:
    alphabet = string.ascii_letters + string.digits + "!@#$%^&*-_"
    pwd = [
        secrets.choice(string.ascii_uppercase),
        secrets.choice(string.ascii_lowercase),
        secrets.choice(string.digits),
        secrets.choice("!@#$%^&*-_"),
    ]
    pwd += [secrets.choice(alphabet) for _ in range(length - 4)]
    secrets.SystemRandom().shuffle(pwd)
    return "".join(pwd)
 
# ─────────────────────────────────────────────
#  STEP 0 — Connectivity
# ─────────────────────────────────────────────
 
def check_connectivity():
    hdr("STEP 0 — Connectivity check")
    try:
        r = requests.get(BASE_URL, timeout=8)
        ok(f"Backend reachable at {BASE_URL}  (HTTP {r.status_code})")
    except requests.exceptions.ConnectionError:
        err(f"Cannot reach {BASE_URL}")
        print("     → Is the server running? Try: cargo run")
        sys.exit(1)
 
# ─────────────────────────────────────────────
#  STEP 1 — Login
#  POST /auth/login
#  Body: {"login": "...", "password": "..."}
#  Response: text/plain raw token string
# ─────────────────────────────────────────────
 
def login() -> dict:
    hdr("STEP 1 — Admin login")
    r = requests.post(
        base_url("/auth/login"),
        json={"login": ADMIN_USER, "password": ADMIN_PASS},
    )
    if r.status_code not in (200, 201):
        err(f"Login failed  HTTP {r.status_code}: {r.text[:200]}")
        sys.exit(1)
 
    token = r.text.strip()
    if not token:
        err("Login returned an empty token.")
        sys.exit(1)
 
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type":  "application/json",
    }
    ok(f"Logged in as '{ADMIN_USER}'  token: {token[:24]}…")
    return headers
 
# ─────────────────────────────────────────────
#  STEP 2 — Create tournaments
#  POST /tournaments
#  Body: {"full_name": "...", "shortened_name": "..."}
#  Both fields are REQUIRED per Tournament schema
# ─────────────────────────────────────────────
 
def create_tournaments(headers: dict) -> dict:
    """Returns {org: [tournament_id, ...]}"""
    hdr("STEP 2 — Creating tournaments")
 
    existing_r       = requests.get(base_url("/tournaments"), headers=headers)
    existing         = existing_r.json() if existing_r.status_code == 200 else []
    # Tournament uses full_name as the unique name field
    existing_by_name = {t.get("full_name"): t.get("id") for t in existing}
 
    org_tournaments: dict = {}
 
    for org in ORGANIZATIONS:
        org_tournaments[org] = []
        for n in range(1, TOURNAMENTS_N + 1):
            full_name      = f"{org}.Tournament{n}"
            shortened_name = f"{org[:3].upper()}T{n}"   # e.g. "DEBT1", "ZSKT2"
 
            if full_name in existing_by_name:
                tid = existing_by_name[full_name]
                org_tournaments[org].append(tid)
                warn(f"{full_name} already exists  (id={tid})")
                continue
 
            r = requests.post(base_url("/tournaments"), headers=headers, json={
                "full_name":      full_name,
                "shortened_name": shortened_name,
            })
            if r.status_code in (200, 201):
                tid = r.json().get("id")
                org_tournaments[org].append(tid)
                ok(f"{full_name}  (id={tid})")
            else:
                err(f"{full_name} — HTTP {r.status_code}: {r.text[:120]}")
 
    return org_tournaments
 
# ─────────────────────────────────────────────
#  STEP 3 — Create user accounts
#  POST /users
#  Body schema: UserWithPassword → {"handle": "...", "password": "..."}
#  Response: User → {"id": "<uuid>", "handle": "...", "picture_link": ...}
# ─────────────────────────────────────────────
 
def create_users(headers: dict) -> dict:
    """Returns {handle: {"id": <uuid str>, "password": str}}"""
    hdr("STEP 3 — Creating user accounts")
 
    existing_r   = requests.get(base_url("/users"), headers=headers)
    existing     = existing_r.json() if existing_r.status_code == 200 else []
    existing_ids = {u.get("handle"): u.get("id") for u in existing}
 
    users: dict = {}
 
    for org in ORGANIZATIONS:
        for role in ROLES:
            handle   = f"{org}.{role}"
            password = generate_password()
 
            if handle in existing_ids:
                uid = existing_ids[handle]
                users[handle] = {"id": uid, "password": password}
                # PATCH /users/{id}/password — field is "new_password" per UserPasswordPatch schema
                patch_r = requests.patch(
                    base_url(f"/users/{uid}/password"),
                    headers=headers,
                    json={"new_password": password},
                )
                if patch_r.status_code in (200, 204):
                    warn(f"{handle} already exists — password reset  (id={uid})")
                else:
                    warn(f"{handle} exists but password reset failed ({patch_r.status_code}): {patch_r.text[:80]}")
                continue
 
            r = requests.post(base_url("/users"), headers=headers, json={
                "handle":   handle,
                "password": password,
            })
            if r.status_code in (200, 201):
                uid = r.json().get("id")
                users[handle] = {"id": uid, "password": password}
                ok(f"{handle}  (id={uid})")
            else:
                err(f"{handle} — HTTP {r.status_code}: {r.text[:120]}")
 
    return users
 
# ─────────────────────────────────────────────
#  STEP 4 — Assign roles
#  POST /users/{user_id}/tournaments/{tournament_id}/roles
#  Body: array of Role strings e.g. ["Marshal"]   ← raw JSON array, NOT {"roles": [...]}
#  Valid Role enum values: "Organizer" | "Judge" | "Marshal"
#  409 = already assigned → use PATCH to overwrite
# ─────────────────────────────────────────────
 
def assign_roles(headers: dict, org_tournaments: dict, users: dict):
    hdr("STEP 4 — Assigning roles in tournaments")
 
    for org in ORGANIZATIONS:
        for role in ROLES:
            handle = f"{org}.{role}"
            user   = users.get(handle)
            if not user:
                err(f"{handle} missing from users dict — skipping")
                continue
 
            uid = user["id"]
            for tid in org_tournaments.get(org, []):
                role_url = base_url(f"/users/{uid}/tournaments/{tid}/roles")
 
                # Try POST first; if 409 (already assigned) use PATCH to overwrite
                r = requests.post(role_url, headers=headers, json=[role])
 
                if r.status_code in (200, 201):
                    ok(f"{handle} → '{role}' in tournament id={tid}")
                elif r.status_code == 409:
                    # Already has roles — overwrite with PATCH
                    pr = requests.patch(role_url, headers=headers, json=[role])
                    if pr.status_code in (200, 201):
                        warn(f"{handle} → '{role}' overwritten in tournament id={tid}")
                    else:
                        err(f"{handle} PATCH roles in tournament id={tid} — HTTP {pr.status_code}: {pr.text[:80]}")
                else:
                    err(f"{handle} in tournament id={tid} — HTTP {r.status_code}: {r.text[:120]}")
 
# ─────────────────────────────────────────────
#  STEP 5 — Verify
# ─────────────────────────────────────────────
 
def verify(headers: dict, users: dict, org_tournaments: dict):
    hdr("STEP 5 — Verification")
 
    # 5a. Tournament count
    r        = requests.get(base_url("/tournaments"), headers=headers)
    all_t    = r.json() if r.status_code == 200 else []
    expected = {f"{o}.Tournament{n}" for o in ORGANIZATIONS for n in range(1, TOURNAMENTS_N + 1)}
    found    = {t.get("full_name") for t in all_t}
    missing  = expected - found
 
    if not missing:
        ok(f"All {len(expected)} tournaments confirmed in API")
    else:
        err(f"Missing tournaments: {missing}")
 
    # 5b. Role assignment check
    print(f"\n  Verifying role assignments…")
    for org in ORGANIZATIONS:
        for role in ROLES:
            handle = f"{org}.{role}"
            user   = users.get(handle)
            if not user:
                continue
            uid    = user["id"]
            all_ok = True
            for tid in org_tournaments.get(org, []):
                gr = requests.get(base_url(f"/users/{uid}/tournaments/{tid}/roles"), headers=headers)
                if gr.status_code == 200:
                    assigned = gr.json()   # returns ["Marshal"] or ["Judge"] etc.
                    if role in assigned:
                        pass  # correct
                    else:
                        err(f"{handle} — role '{role}' not found in tournament id={tid}, got: {assigned}")
                        all_ok = False
                else:
                    err(f"{handle} — HTTP {gr.status_code} for tournament id={tid}")
                    all_ok = False
            if all_ok:
                ok(f"{handle} — '{role}' confirmed in all {TOURNAMENTS_N} tournaments")
 
    # 5c. Login test for every account
    print(f"\n  {'Handle':<32} {'Login test'}")
    print(f"  {'─'*32} {'─'*16}")
    for handle, info in users.items():
        lr = requests.post(base_url("/auth/login"), json={
            "login":    handle,
            "password": info["password"],
        })
        if lr.status_code in (200, 201):
            result = f"{GREEN}✓ OK{RESET}      (HTTP {lr.status_code})"
        else:
            result = f"{RED}✗ FAILED{RESET}  (HTTP {lr.status_code})"
        print(f"  {handle:<32} {result}")
 
# ─────────────────────────────────────────────
#  STEP 6 — Save credentials
# ─────────────────────────────────────────────
 
def save_credentials(users: dict):
    hdr("STEP 6 — Credentials")
 
    creds = {handle: info["password"] for handle, info in users.items()}
 
    print(f"\n  {'Handle':<32} {'Password'}")
    print(f"  {'─'*32} {'─'*22}")
    for handle, password in creds.items():
        print(f"  {handle:<32} {password}")
 
    with open("credentials.json", "w") as f:
        json.dump(creds, f, indent=2)
 
    print(f"""
  {GREEN}Saved to credentials.json{RESET}
 
  ┌──────────────────────────────────────────────────────┐
  │  Next steps:                                         │
  │  1. Paste the credentials table above as a comment   │
  │     in your task file.                               │
  │  2. Share credentials.json securely with testers.    │
  │  3. Delete credentials.json when done.               │
  └──────────────────────────────────────────────────────┘""")
 
# ─────────────────────────────────────────────
#  MAIN
# ─────────────────────────────────────────────
 
def main():
    print(f"\n{BOLD}{'═'*55}")
    print(f"  Granda Tournament Setup Script")
    print(f"  Target: {BASE_URL}")
    print(f"{'═'*55}{RESET}")
 
    check_connectivity()
    headers         = login()
    org_tournaments = create_tournaments(headers)
    users           = create_users(headers)
    assign_roles(headers, org_tournaments, users)
    verify(headers, users, org_tournaments)
    save_credentials(users)
 
    print(f"\n{GREEN}{BOLD}All done.{RESET}\n")
 
 
if __name__ == "__main__":
    main()
 