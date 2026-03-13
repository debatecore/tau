# Tau - the debate planner response cannon

Tau is the debatecore debate tournament planner project's response cannon - also known as a backend.

## Deployment and local development

The suggested way to develop and deploy requires you to use docker and cargo.
You may also venture on your own, adapting the following instructions to your needs:

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
