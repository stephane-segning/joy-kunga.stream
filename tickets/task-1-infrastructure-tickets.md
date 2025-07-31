# Task #1 Infrastructure GitHub Tickets

This document contains all the GitHub tickets created for Task #1: Setup Project Infrastructure and Database Schema.

## Main Epic Issue

### Issue #1: Setup Project Infrastructure and Database Schema
- **URL**: https://github.com/stephane-segning/joy-kunga.stream/issues/1
- **Type**: Epic/User Story
- **Labels**: epic, infrastructure, high-priority, backend
- **Status**: Open

**User Story**: As a developer, I want to set up the foundational infrastructure including PostgreSQL database schema, Redis setup, and core project structure for the Rust backend services, so that I can build upon a solid foundation for the streaming platform.

**Acceptance Criteria**:
- [ ] PostgreSQL 17+ database is provisioned with proper schema
- [ ] Database includes User table {id, email, provider, roles, settings}
- [ ] Database includes MediaItem table {id, type, metadata, s3Key, status}
- [ ] Database includes Session table {token, userId, expiresAt}
- [ ] Redis 6+ is set up for caching and session storage
- [ ] Rust workspace is initialized with Tokio async runtime (Rust edition 2024)
- [ ] Database migrations are configured using sqlx
- [ ] Connection pooling is implemented
- [ ] Basic error handling is in place

---

## Subtask Issues

### Issue #2: Initialize Rust Workspace and Project Structure
- **URL**: https://github.com/stephane-segning/joy-kunga.stream/issues/2
- **Type**: Subtask of #1
- **Labels**: subtask, infrastructure, rust, setup
- **Status**: Open
- **Dependencies**: None (foundation task)

**Description**: Set up the core Rust monorepo workspace using Cargo, configured with the Tokio async runtime and Rust 2024 edition.

**Acceptance Criteria**:
- [ ] Create a new Cargo workspace
- [ ] Configure the root `Cargo.toml` to specify the 2024 edition
- [ ] Add `tokio` as a core dependency with 'full' features
- [ ] Establish a basic directory structure for future microservices (e.g., `services/`, `libs/`)

**Test Strategy**: Verify the project compiles successfully using `cargo check` and `cargo build`.

---

### Issue #3: Provision PostgreSQL and Define Schema with Migrations
- **URL**: https://github.com/stephane-segning/joy-kunga.stream/issues/3
- **Type**: Subtask of #1
- **Labels**: subtask, infrastructure, database, postgresql, migrations
- **Status**: Open
- **Dependencies**: None (can be done in parallel)

**Description**: Set up a PostgreSQL 17+ instance and use `sqlx-cli` to define and apply the initial database schema for User, MediaItem, and Session tables.

**Acceptance Criteria**:
- [ ] Provision a PostgreSQL 17+ server (e.g., using Docker)
- [ ] Install `sqlx-cli`
- [ ] Initialize migrations
- [ ] Create the first migration file defining the required tables
- [ ] Define `User` table {id, email, provider, roles, settings}
- [ ] Define `MediaItem` table {id, type, metadata, s3Key, status}
- [ ] Define `Session` table {token, userId, expiresAt}
- [ ] Use appropriate data types (e.g., UUID, TEXT, JSONB, TIMESTAMPTZ)

**Test Strategy**: Apply the migration using `sqlx migrate run`. Manually verify the table structures and constraints using a SQL client.

---

### Issue #4: Implement PostgreSQL Connection Pooling in Rust
- **URL**: https://github.com/stephane-segning/joy-kunga.stream/issues/4
- **Type**: Subtask of #1
- **Labels**: subtask, infrastructure, database, rust, sqlx, connection-pooling
- **Status**: Open
- **Dependencies**: 
  - Depends on #2 (Initialize Rust Workspace and Project Structure)
  - Depends on #3 (Provision PostgreSQL and Define Schema with Migrations)

**Description**: Integrate the `sqlx` crate into the Rust project to establish a connection pool to the PostgreSQL database and implement basic data access logic.

**Acceptance Criteria**:
- [ ] Add `sqlx` with `postgres` and `runtime-tokio-rustls` features to project dependencies
- [ ] Create a database module to manage the `PgPool` instance
- [ ] Configure connection pool via environment variables
- [ ] Implement a basic health check function to verify database connectivity
- [ ] Define custom error types for database operations

**Test Strategy**: Write a unit test that successfully acquires a connection from the pool and performs a simple query (e.g., `SELECT 1`). Test connection failure scenarios.

---

### Issue #5: Provision Redis and Implement Rust Client
- **URL**: https://github.com/stephane-segning/joy-kunga.stream/issues/5
- **Type**: Subtask of #1
- **Labels**: subtask, infrastructure, redis, rust, caching
- **Status**: Open
- **Dependencies**: 
  - Depends on #2 (Initialize Rust Workspace and Project Structure)

**Description**: Set up a Redis 6+ instance and integrate a Redis client into the Rust application for future caching and session storage.

**Acceptance Criteria**:
- [ ] Provision a Redis 6+ server (e.g., using Docker)
- [ ] Add the `redis` crate with the `tokio-comp` feature to the project
- [ ] Create a cache module to manage the Redis connection
- [ ] Configure Redis connection via environment variables
- [ ] Implement basic `get` and `set` wrapper functions

**Test Strategy**: Write an integration test to connect to Redis, set a key-value pair with a TTL, retrieve it, and verify it is deleted after the TTL expires.

---

### Issue #6: Create Initial Infrastructure Integration Tests
- **URL**: https://github.com/stephane-segning/joy-kunga.stream/issues/6
- **Type**: Subtask of #1
- **Labels**: subtask, infrastructure, testing, integration-tests, rust
- **Status**: Open
- **Dependencies**: 
  - Depends on #4 (Implement PostgreSQL Connection Pooling in Rust)
  - Depends on #5 (Provision Redis and Implement Rust Client)

**Description**: Develop a suite of integration tests to validate that the Rust application can connect to both PostgreSQL and Redis, and that the database schema is correctly set up.

**Acceptance Criteria**:
- [ ] Create an integration test that initializes both the PostgreSQL connection pool and the Redis client
- [ ] Test should perform a simple write/read operation against a test table in the database
- [ ] Test should perform a SET/GET operation in Redis
- [ ] Confirm both services are reachable and operational from the application
- [ ] Tests should run in isolation and clean up after themselves

**Test Strategy**: Run the integration test suite using `cargo test`. Success is defined by all tests passing, confirming the foundational infrastructure is correctly configured and accessible.

---

## Dependency Graph

```
#1 (Epic) - Setup Project Infrastructure and Database Schema
├── #2 - Initialize Rust Workspace and Project Structure (no dependencies)
├── #3 - Provision PostgreSQL and Define Schema with Migrations (no dependencies)
├── #4 - Implement PostgreSQL Connection Pooling in Rust (depends on #2, #3)
├── #5 - Provision Redis and Implement Rust Client (depends on #2)
└── #6 - Create Initial Infrastructure Integration Tests (depends on #4, #5)
```

## Implementation Order

Based on dependencies, the recommended implementation order is:

1. **Parallel Start**: Issues #2 and #3 can be started simultaneously
2. **After #2 and #3**: Issue #4 can begin
3. **After #2**: Issue #5 can begin (parallel with #4)
4. **After #4 and #5**: Issue #6 (final integration tests)

## Notes

- All tickets have been successfully created in the GitHub repository
- Each subtask includes detailed acceptance criteria and test strategies
- Labels have been applied for easy filtering and organization
- The main epic (#1) should be updated to reference all subtask issues for better tracking

## Next Steps

1. Manually update issue #1 to include references to subtask issues #2-#6
2. Begin implementation starting with issues #2 and #3
3. Update issue status as work progresses
4. Close subtask issues as they are completed
5. Close the main epic (#1) when all subtasks are complete