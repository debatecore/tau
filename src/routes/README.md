# routes

1. Files in this directory implement a public fn `route()` by convention. These are subsequently merged into a router that handles all API routes in `mod.rs`;
2. All `route()` functions must be documented using [utoipa](https://docs.rs/utoipa/4.2.3). For the documentation to be included in `/swagger-ui`, modify the ApiDoc declaration in `swagger.rs`.
