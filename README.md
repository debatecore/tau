# Tau - the debate planner response cannon

Tau is the debatecore debate tournament planner project's response cannon - also known as a backend.

## Deployment and local development
The easiest way to develop locally is to compile, test, and run using the cargo suite of tools and
set up the database with docker compose and sqlx-cli.

To set up the database for local development, run: `docker compose --profile dev up -d`. Be sure to run the
migrations with `sqlx migrate run` when you create the database. When you're ready to disable it,
run `docker compose --profile dev down`. You can restart its state by deleting the `tau_dbdevdata` docker
volume when it's off.

To deploy the project, you can use the other available profile: `--profile prod`.
This will build and run both the backend and the database in a production-ready way.

## Documentation
Once the project is built, you can access the API documentation at [localhost:2023/swagger-ui](http://localhost:2023/swagger-ui).
