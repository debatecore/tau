# Tau - the debate planner response cannon

Tau is the debatecore debate tournament planner project's response cannon - also known as a backend.

## Deployment and local development

The suggested way to develop and deploy requires you to use docker and cargo.

### Local development
Set the following environment variables, via `.env` or your shell:
- `DOCKER_DB_ROOT_PASSWORD` will be used as the password for the database root user.
- `DATABASE_URL` is used for db connection. During development, this is `postgres://tau:tau@localhost:5432/tau`.
- `SECRET` will be used as high entropy data used for generating tokens.

Start the database with `docker compose --profile dev (up -d/down)`.

Run the migrations via sqlx-cli with `sqlx run migrate` or via other means.
You can reset the database by deleting the `tau_dbdevdata` docker volume when it's off.

Compile and run the project with `cargo`.

### Deployment
For deploying via docker, set the aforementioned environment variables as well as:
- `DOCKER_DB_PASSWORD` will be used as the password for the backend's database role.
Then, run `docker compose --profile prod`.

## Documentation
Once the project is built, you can access the API documentation at [localhost:2023/swagger-ui](http://localhost:2023/swagger-ui).
