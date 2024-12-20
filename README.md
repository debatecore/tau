# Tau - the debate planner response cannon

Tau is the debatecore debate tournament planner project's response cannon - also known as a backend.

## Deployment and local development

### Local development
The easiest way to develop locally is via a combination of docker and cargo - using docker to handle the database and cargo to handle the rest.
This setup requires a few environment variables:
- `DOCKER_DB_ROOT_PASSWORD` will be used as the password for the database root user.
- `DATABASE_URL` will be used to connect to the database. During development, this is `postgres://tau:tau@localhost:5432/tau` by default.
You may create a `.env` file in the root of the project or just set the environment variables in your shell.

Next comes the database, which you can turn on with `docker compose --profile dev up -d`. You can run the migrations with `sqlx migrate run` to set up the database.
When you're done, you can turn it off with `docker compose --profile dev down`. To reset the database state, you can delete the `tau_dbdevdata` docker volume when it's off.

Finally, you can run the backend with `cargo run`. The backend will be available at `localhost:2023`. Yo can override it by setting the `PORT` environment variable.

### Deployment
For deploying, you will need the aforementioned environment variables, as well as a new one:
- `DOCKER_DB_PASSWORD` will be used as the password for the tau database user.
You can again set them in a `.env` file or in your shell.

To take the project online, you can use the other available compose profile by running `docker compose --profile prod up -d`.
This will build and run both the backend and the database in a production-ready way.

## Documentation
Once the project is built, you can access the API documentation at [localhost:2023/swagger-ui](http://localhost:2023/swagger-ui).
