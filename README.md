# RTC :world_map:

_Note: currently a work in progress_

Real Time Cartographer is a tool that enables developers to visualize their cloud infrastructure.

## Goal

The goal with this project is to generate a graph based off of the GCP log entries representing their architecture.

Here would be an example of a visual the graph database could potentially show.

```mermaid
---
config:
      theme: redux
---
flowchart TD
        App(["App"])
        GraphQL(["GraphQL Server"])
        Users(["Users Service"])
        Books(["Books Service"])
        Auth(["Authentication Service"])
        App -->|"Mutation login"| GraphQL
        GraphQL -->|"POST /login"| Auth
        GraphQL -->|"POST /validate_token"| Auth
        App -->|"Query user"| GraphQL
        GraphQL -->|"GET /users/:id"| Users
        App -->|"Mutation create book"| GraphQL
        GraphQL -->|"POST /books"| Books
        Books --> |"GET /roles"| Users
        Auth --> |"GET /users/:id"| Users
```

## Local

### Requirements

1. docker-compose or podman
2. `rtc.toml` to configure (see `rtc.example.toml`)

### Building

```sh
cargo build
```

### Running

```sh
cargo run -- help
RUST_LOG=debug cargo run -- run
RUST_LOG=debug cargo run -- demo
```
