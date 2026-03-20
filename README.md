# Tau - the debate planner response cannon

Tau is the debatecore debate tournament planner project's response cannon - also known as a backend.

## Deployment and local development
The suggested way to develop and deploy requires you to use docker and cargo.
You may also venture on your own, adapting the following instructions to your needs:

### Local Development Guide
This guide walks you through setting up "tau" backend locally for development on Windows, MacOS and Linux.

### Table of Contents
-> Prerequisites
-> Step 1 - GitHub Setup 
-> Step 2 - Clone the Repository
-> Step 3 - Install Rust
-> Step 4 - Install sqlx-cli
-> Step 5 - Install Docker Desktop
-> Step 6 - Configure Environment
-> Step 7 - Start the Database
-> Step 8 - Run Migrations
-> Step 9 - Run the Backend
-> Step 10 - Setup the Git Hook
-> Daily Workflow
-> Understanding Docker Containers


Prerequisites

Tool               Purpose

Git              Version control
Rust + Cargo     Compiling and running the backend
sqlx-cli         Running database migrations
Docker Desktop   Running the local database container



### Step 1 — GitHub Setup

Create a GitHub account if you don't have one.
Post your GitHub username in the issue comment to receive a repository invitation.
Accept the invitation via your email or github.com/notifications.


Optional (if new to Git): After cloning, create and push a test branch to verify write access:

bash
git checkout -b test/your-name
git push origin test/your-name



### Step 2 — Clone the Repository

bash
git clone https://github.com/debatecore/tau.git
cd tau

Windows users: Before cloning, disable automatic line ending conversion to prevent shell script corruption inside Docker:

cmd
git config --global core.autocrlf false



### Step 3 — Install Rust
macOS / Linux

bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

Restart your terminal, then verify:
bash
cargo --version
rustc --version

Windows

Download and run the installer from rustup.rs — it will be a file called rustup-init.exe.
Press Enter to accept the default installation (option 1).
Close and reopen your terminal after installation.

Windows note: Rust requires the MSVC C++ build tools. If the installer warns you about this, follow its link to install Visual Studio Build Tools with the "Desktop development with C++" workload, then re-run the Rust installer.

Verify:
cmd
cargo --version



### Step 4 - Install sqlx-cli

This tool runs the database migrations.

bash
cargo install sqlx-cli

This works the same on Windows, macOS, and Linux.



### Step 5 — Install Docker Desktop

Docker Desktop runs the PostgreSQL database in an isolated container.

OS                       Instructions   

macOS                    Download from docker.com/products/docker-desktop — install the .dmg and launch Docker from Applications

Linux                    Follow the official Linux installation guide for your distro

Windows                  Download from docker.com/products/docker-desktop — run the installer and restart if prompted
                         
                  
After installation, start Docker Desktop and wait until it shows "Engine running" before continuing.



### Step 6 — Configure Environment

Create a .env file in the project root.

macOS / Linux

bash
cp .env.example .env   # if an example file exists, otherwise:
touch .env
nano .env              # or use any text editor

Windows (Command Prompt)

cmd
echo. > .env
notepad .env

Paste the following into .env and save:
env
DATABASE_URL=postgres://tau:tau@localhost:5432/tau
SECRET=CENTRUMRWLYSONOSTARPOZNANCDNSBCD4L52SPM
DOCKER_DB_ROOT_PASSWORD=superdoopersecretpasswordthatcannotbeleaked
DOCKER_DB_PASSWORD=wedoingsecurityinhere
FRONTEND_ORIGIN=http://localhost:3000
PORT=2023

You can change SECRET, DOCKER_DB_ROOT_PASSWORD, and DOCKER_DB_PASSWORD to any values you like. The DATABASE_URL must remain exactly as shown for local development.



### Step 7 — Start the Database

This starts a PostgreSQL database container using the dev Docker Compose profile.

bash
docker compose --profile dev up -d

Verify the container is running:

bash
docker ps

You should see a container named tau-db-dev-1 with status Up.

Troubleshooting: If the container starts but the tau user is missing (visible in docker logs tau-db-dev-1), the init script may have failed. On Windows this is caused by CRLF line endings — see the fix below. On macOS/Linux this should not occur.

Windows fix: CRLF line endings in shell scripts

Open PowerShell in the project directory and run:
powershell
(Get-Content "dbinit-dev.sh" -Raw) -replace "`r`n", "`n" | Set-Content "dbinit-dev.sh" -NoNewline
(Get-Content "dbinit.sh" -Raw) -replace "`r`n", "`n" | Set-Content "dbinit.sh" -NoNewline

Then restart the container from scratch:
cmddocker compose --profile dev down -v
docker compose --profile dev up -d



### Step 8 — Run Migrations

Wait ~10 seconds after starting the container for PostgreSQL to fully initialize, then run:

bash
sqlx migrate run

You should see output confirming migrations were applied. This only needs to be done once (and again when new migrations are added to the repo).



### Step 9 — Run the Backend

bash
cargo run

The first build will take a few minutes as Rust compiles all dependencies. Subsequent runs are much faster. A successful start looks like this:
INFO tau::setup: Response cannon spinning up...
INFO tau::setup: Loaded .env
INFO tau::database: Connection with the database successful
INFO tau::database: Database migrations successful.
INFO tau::setup: Listener socket address is: 0.0.0.0:2023

Important: The database container must be running before cargo run. The sqlx package validates SQL queries at compile time, so the build will fail if the database is unreachable.



### Step 10 — Set Up the Git Hook

Run this once to keep the .sqlx query cache up to date before each commit:

bash
git config --local core.hooksPath .githooks/

No output means it worked. Verify with:

bash
git config --local core.hooksPath
Should print: .githooks/



### Daily Workflow

Starting development

bash
1. Start Docker Desktop (if not already running)
2. Start the database
docker compose --profile dev up -d


#3. Run the backend
cargo run
#Stopping when done
bash
Stop the backend: press Ctrl+C in the cargo run terminal


#Stop the database
docker compose --profile dev down

#Optionally: quit Docker Desktop from the system tray / menu 



### Understanding Docker Containers

The problem Docker solves

When developing in a team, everyone's machine is different — different operating systems, different software versions, different configurations. Without a shared environment, a database that works perfectly on one machine may behave differently or fail to install on another. This is the classic "it works on my machine" problem.
Docker solves this by packaging software — along with everything it needs to run — into a container: a standardized, isolated unit that behaves identically on any machine that has Docker installed.

What a container actually is

Think of a container as a lightweight, self-contained computer running inside your computer. It has its own:

Filesystem
Network interface
Running processes
Environment variables

However, unlike a full virtual machine, containers share the host machine's operating system kernel, making them fast to start and cheap on resources.


Images vs Containers

A Docker image is a read-only blueprint — like a class in programming. A container is a running instance of that image — like an object instantiated from the class. You can run many containers from the same image simultaneously.
In this project, the image is postgres:17.2 (downloaded automatically from Docker Hub), and tau-db-dev-1 is the container running from it.

Volumes — persisting data

Containers are ephemeral by default: when you stop and remove one, all data inside it is gone. Volumes are the solution — they are storage areas that exist outside the container and survive restarts.
In this project, tau_dbdevdata is the volume that persists your database's data. This is why docker compose down (without -v) keeps your data safe, while docker compose down -v wipes it completely (useful for a clean reset).

Docker Compose and profiles

Docker Compose is a tool for defining and running multi-container setups using a compose.yaml file. Instead of typing long docker run commands with many flags, you declare your services, networks, and volumes in one file.
Profiles let you group services for different scenarios. This project uses two:

Profile                Command                          Purpose   

dev           docker compose --profile dev up -d     Starts only the database; you run the backend locally with cargo run
prod          docker compose --profile prod up       Starts both the database AND the backend in containers (for deployment)

During local development you use --profile dev because you want to run the backend yourself (for fast recompilation, debugging, etc.) while keeping the database containerized for consistency.

The init script

When the PostgreSQL container starts for the very first time, it runs any .sh scripts found in /docker-entrypoint-initdb.d/. In this project that script (dbinit-dev.sh) creates the tau database user and the tau database with the credentials defined in your .env file. This only runs once — on subsequent starts the database directory already exists and initialization is skipped.

This is why the order matters: the .env file must exist before you first start the container, and if something goes wrong during initialization, you need docker compose down -v to delete the volume and trigger re-initialization from scratch.

### Local development
Set the following environment variables, via `.env` or your shell:
- `DOCKER_DB_ROOT_PASSWORD` will be used as the password for the database root user.
- `DATABASE_URL` is used for db connection. During development, this is `postgres://tau:tau@localhost:5432/tau`.
- `FRONTEND_ORIGIN` will be used as an allowed [origin](https://developer.mozilla.org/en-US/docs/Glossary/Origin) for the purpose of [CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS). Must be a valid URL.

Start the database with `docker compose --profile dev (up -d/down)`.
Run the migrations via sqlx-cli with `sqlx migrate run` or by other means.

Compile and run the project with `cargo`.

#### `sqlx` preparation hook

It is advisable to run `git config --local core.hooksPath .githooks/` to have the `.sqlx` directory updated before making a commit.

### Deployment
For deploying via docker, set the following environment variables:
- `DOCKER_DB_PASSWORD` which will be used as the password for the backend's database access user.
- `DOCKER_DB_ROOT_PASSWORD` will be used as the password for the database root user.
- `FRONTEND_ORIGIN` will be used as an allowed [origin](https://developer.mozilla.org/en-US/docs/Glossary/Origin) for the purpose of [CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS). Must be a valid URL (`http://localhost:3000` by default).
Then, run `docker compose --profile prod`.

### Optional configuration
- `SECRET` will be used as additional high entropy data used for generating tokens. By default, tau uses system entropy and the current UNIX timestamp.
- `PORT` will be used as the port the server listens on. The default is 2023.

The following example `.env` file is geared for both scenarios:
```env
DATABASE_URL=postgres://tau:tau@localhost:5432/tau
SECRET=CENTRUMRWLYSONOSTARPOZNANCDNSBCD4L52SPM
DOCKER_DB_ROOT_PASSWORD=superdoopersecretpasswordthatcannotbeleaked
DOCKER_DB_PASSWORD=wedoingsecurityinhere
FRONTEND_ORIGIN=https://example.com
PORT=2019
```

## Documentation
Once the project is built, you can access the API documentation at [localhost:2023/swagger-ui](http://localhost:2023/swagger-ui).
