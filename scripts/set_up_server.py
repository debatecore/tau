import requests
import secrets
import string
import json
import sys
from typing import Optional


BASE_URL = "https://granda/backend.debateco.re"
ADMIN_USER = "admin"
ADMIN_PASS = "even&headless&fastball&wager&reprise&unblessed&scorebook"

ORGANIZATIONS = ["DebateLab", "ZSK", "Buster"]
ROLES = ["Organizer", "Judge", "Marshal"]

TOURNAMENTS_PER_ORG = 3

BOLD  = "\033[1m"
GREEN = "\033[92m"
RED   = "\033[91m"
CYAN  = "\033[96m"
YELLOW= "\033[93m"
RESET = "\033[0m"
 
def ok(msg):   print(f"  {GREEN}✓{RESET} {msg}")
def err(msg):  print(f"  {RED}✗{RESET} {msg}")
def warn(msg): print(f"  {YELLOW}!{RESET} {msg}")
def hdr(msg):  print(f"\n{BOLD}{CYAN}{'─'*55}\n  {msg}\n{'─'*55}{RESET}")

def url(path: str) -> str:
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

# step 0 - connectivitiy check

def chekc_connectivity():
    hdr("step 0 - connectivity check")
    try:
        r = requests.get(BASE_URL, timeout=8)
        ok(f"backend reachable at {BASE_URL}  (HTTP{r.status_code})")

    except requests.exceptions.ConnectionError:
        err(f"backend not reachable at {BASE_URL}")
        print("     → Check your network connection or VPN.")
        sys.exit(1)
   

# step 1 - login 
# POST /auth/login

def login() -> dict:
    hdr("step 1 - admin login")
    r = requests.post(url("/auth/login"),json={"handle": ADMIN_USER, "password": ADMIN_PASS})

    if r.status_code not in (200,201):
        err(f"login failed HTTP{r.status_code}: {r.text[:200]}")
        sys.exit(1)
    
    token = r.text.strip()
    if not token:
        err(f"login returned empty token")
        sys.exit(1)
    
    headers = {
        "authorization": f"Bearer {token}",
        "content-type": "application/json"
    }
    
    ok(f"Logged in as {ADMIN_USER}, token:{token[:24]}...")
    return headers


# step 2 - create tournaments 
# POST /tournaments, GET /tournaments

def create_tournaments(headers: dict) -> dict:
    hdr("step 2 - create tournaments")
    existing_r = requests.get(url("/tournaments"), headers=headers)
    existing = existing_r.json() if existing_r.status_code == 200 else []

    existing_by_name = {t.get("name"): t.get("id") or t.get("pk") for t in existing}

    org_tournaments = dict[str, list] = {}

    for org in ORGANIZATIONS:
        org_tournaments[org] = []
        for n in range(1, TOURNAMENTS_PER_ORG + 1):
            name = f"{org} Tournament {n}"

            if name in existing_by_name:
                t_id = existing_by_name[name]
                org_tournaments[org].append(t_id)
                warn(f"{name}' already exists with id {t_id}.")
                continue

            r = requests.post(url("/tournaments"), headers=headers, json={"name": name})
            if r.status_code in (200, 201):
                data = r.json()
                t_id = data.get("id") or data.get("pk")
                org_tournaments[org].append(t_id)
                ok(f"{name} (id={t_id})")
            else:
                err(f"Failed to create {name} (HTTP{r.status_code}): {r.text[:200]}")
                
    return org_tournaments


# step 3 - create user accounts
# POST /users, GET /users

def create_users(headers:dict) -> dict:
    hdr("step 3 - create user accounts")
    existing_r = requests.get(url("/users"), headers=headers)
    existing = existing_r.json() if existing_r.status_code == 200 else []

    existing_ids = {u.get("handle"): u.get("id") for u in existing}

    users: dict[str, dict] = {}

    for org in ORGANIZATIONS:
        for role in ROLES:
            handle = f"{org}.{role}"
            password = generate_password()

            if handle in existing_ids:
                u_id = existing_ids[handle]
                users[handle] = {"id": u_id, "password": password}
                # reset password so we have known value
                patch_r = requests.patch(
                    url(f"/users/{u_id}/password"), headers=headers, json={"password": password},
                )
                if patch_r.status_code in (200, 204):
                    warn(f"User '{handle}' already exists (id={u_id}), password reset.")
                else:
                    err(f"Failed to reset password for '{handle}' (HTTP{patch_r.status_code}): {patch_r.text[:200]}")
                continue

            # POST /users  body:{handle, password}
            r = requests.post(
                url("/users"), headers=headers, json={"handle": handle, "password": password}
            )
            if r.status_code in (200, 201):
                data = r.json()
                u_id = data.get("id")
                users[handle] = {"id": u_id, "password": password}
                ok(f"User '{handle}' created (id={u_id})")
            else:
                err(f"Failed to create user '{handle}' (HTTP{r.status_code}): {r.text[:200]}")
        
    return users


# step 4 - assign roles to users in tournaments

def assign_roles(headers:dict, users:dict, org_tournaments:dict):
    hdr("step 4 - assign roles to users in tournaments")
    for org in ORGANIZATIONS:
        for role in ROLES:
            handle = f"{org}.{role}"
            user = users.get(handle)
            if not user:
                err(f"User '{handle}' not found, skipping role assignment.")
                continue

            uuid = user["id"]  # uuid string
            for t_id in org_tournaments.get(org, []):
                role_url = url(f"users/{uuid}/tournaments/{t_id}/roles")
            
            # check if role already assigned
            get_r = requests.get(role_url, headers=headers)
            if get_r.status_code == 200:
                current_roles = get_r.json()
                assigned = (
                    current_roles if isinstance(current_roles, list) else current_roles.get("roles", [])
                )

            if role in assigned or any(
                r.get("name") == role for r in assigned if isinstance(r,dict)):
                warn(f"User '{handle}' already has role '{role}' in tournament {t_id}.")

                continue
            
            r = requests.post(role_url, headers=headers, json={"role": role})
            if r.status_code in (200,201,204):
                ok(f"Assigned role '{role}' to user '{handle}' in tournament {t_id}.")
            else:
                err(f"Failed to assign role '{role}' to user '{handle}' in tournament {t_id} (HTTP{r.status_code}): {r.text[:200]}")
         
                   
# step 5 - verify 

def verify_setup(headers:dict, users:dict, org_tournaments:dict):
    hdr("step 5 - verify setup")
    # tournament check 
    r = requests.get(url("/tournaments"), headers=headers)
    all_tournaments = r.json() if r.status_code == 200 else []
    expected_names = {
        f"{org} Tournament {n}"
        for org in ORGANIZATIONS
        for n in range(1, TOURNAMENTS_PER_ORG + 1)
    }
    found_names = {t.get("name") for t in all_tournaments}
    missing = expected_names - found_names

    if not missing:
        ok(f"all {len(expected_names)} tournaments confirmed.")
    else:
        err(f"Missing tournaments: {', '.join(missing)}")
    
    # role check
    print("\n  Verifying user roles in tournaments...")
    for org in ORGANIZATIONS:
        for role in ROLES:
            handle = f"{org}.{role}"
            user = users.get(handle)
            if not user:
                continue

            uuid = user["id"]  # uuid string
            for t_id in org_tournaments.get(org, []):
                role_url = url(f"users/{uuid}/tournaments/{t_id}/roles")
                gr = requests.get(role_url, headers=headers)
                if gr.status_code != 200:
                    err(f"{handle} - could not fetch the roles for tournament {t_id}")
                    all_ok = False

                if all_ok:
                    ok(f"{handle} — roles confirmed in all {TOURNAMENTS_PER_ORG} tournaments")
     
    
    # login test for every created account
    print(f"\n  {'Handle':<30} {'Login test'}")
    print(f"  {'─'*30} {'─'*14}")
    for handle, info in users.items():
        lr = requests.post(url("/auth/login"), json={
            "handle":   handle,
            "password": info["password"],
        })
        if lr.status_code in (200, 201):
            status = f"{GREEN}✓ OK{RESET}      (HTTP {lr.status_code})"
        else:
            status = f"{RED}✗ FAILED{RESET}  (HTTP {lr.status_code})"
        print(f"  {handle:<30} {status}")
    

# step 6 - save credentials to file

def save_credentials(users: dict):
    hdr("STEP 6 — Credentials")
 
    creds_output = {u: d["password"] for u, d in users.items()}
 
    col_w = 30
    print(f"\n  {'Handle':<{col_w}} {'Password'}")
    print(f"  {'─'*col_w} {'─'*22}")
    for username, password in creds_output.items():
        print(f"  {username:<{col_w}} {password}")
 
    with open("credentials.json", "w") as f:
        json.dump(creds_output, f, indent=2)
 
    print(f"""
  {GREEN}Saved to credentials.json{RESET}""")
    

def main():
    print(f"\n{BOLD}{'═'*55}")
    print(f"  Granda Tournament Setup Script")
    print(f"  Target: {BASE_URL}")
    print(f"{'═'*55}{RESET}")
 
    check_connectivity() # type: ignore
    headers = login()
    org_tournaments = create_tournaments(headers)
    users = create_users(headers)
    assign_roles(headers, org_tournaments, users)
    verify(headers, users, org_tournaments) # type: ignore
    save_credentials(users)
 
    print(f"\n{GREEN}{BOLD}All done.{RESET}\n")
 
if __name__ == "__main__":
    main()
 
    
        











































































































































