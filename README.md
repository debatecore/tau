# Tau - the debate planner response cannon

Tau is the debatecore debate tournament planner project's response cannon - also known as a backend.

## Deployment and local development

### Environment setup
Set environment variables via `.env` or your shell. The following example `.env` file is geared for both production and development:

```env
DATABASE_URL=postgres://tau:tau@localhost:5432/tau
SECRET=CENTRUMRWLYSONOSTARPOZNANCDNSBCD4L52SPM
DOCKER_DB_ROOT_PASSWORD=superdoopersecretpasswordthatcannotbeleaked
DOCKER_DB_PASSWORD=wedoingsecurityinhere
FRONTEND_ORIGIN=https://example.com
PORT=2023
```

#### Required variables

- `DOCKER_DB_ROOT_PASSWORD` will be used as the password for the database root user.
- `DATABASE_URL` is used for db connection. During development, this is `postgres://tau:tau@localhost:5432/tau`.
- `FRONTEND_ORIGIN` will be used as an allowed [origin](https://developer.mozilla.org/en-US/docs/Glossary/Origin) for the purpose of [CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS). Must be a valid URL.

#### Optional configuration
- `SECRET` will be used as additional high entropy data used for generating tokens. By default, tau uses system entropy and the current UNIX timestamp.
- `PORT` will be used as the port the server listens on. The default is 2023.

### Local development

In this scenario it is assumed, that you run the project on your local machine and use a database container for compile-time queries validation.

**Note:** `sqlx` validates queries at compile time, so **being connected to a database is required to compile the crate**.


```mermaid
architecture-beta
    group your_machine(server)[Your machine]
	
		service backend_container(server)[Backend] in your_machine
	group docker[Docker] in your_machine
	    service db_container(database)[Database] in docker
	backend_container:R -- L:db_container
	
	service env(disk)[env file or shell variables] in your_machine
	env:T -- B:backend_container
	env:T -- B:db_container

```

**Prerequisites:** docker,  cargo (Rust), uv (Python).

Once you [configure your environment](#environment-setup), you can run (in your project directory):

```sh
docker compose --profile dev up -d  		# To start the database container
cargo install sqlx-cli && sqlx migrate run  # To perform sqlx migrations (recommended way)
cargo run                           		# To run the crate
```

#### Commit hook

It is advisable to run `git config --local core.hooksPath .githooks/` to configure commit hooks included in this repository.

**Note:** the hooks require you to be connected to the database when committing.

### Deployment

In this scenario, you run fully functional backend split into two containers: first with a database and second with a server.


```mermaid
architecture-beta
    group your_machine(server)[Your machine]
	
	group docker[Docker] in your_machine
	    service backend_container(server)[Backend] in docker
	    service db_container(database)[Database] in docker
	    
	backend_container:R -- L:db_container

	service env(disk)[env file or shell variables] in your_machine
	env:T -- B:backend_container
	env:T -- B:db_container

```

**Prerequisites:** docker.

Once you [configure your environment](#environment-setup), you can simply run:

```
docker compose --profile prod
```

## Documentation
- **API documentation:** Once the project is built, you can access the API documentation at [/swagger-ui](http://localhost:2023/swagger-ui).
- **ER diagram:** a database ER diagram can be found in `docs/er_diagram.pdf`. If you have your [commit hooks](#commit-hook) configured, it will be automatically updated.