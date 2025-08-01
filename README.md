# Joy Kunga Stream

A microservices-based media streaming platform built with Rust, PostgreSQL, and Redis.

## Overview

Joy Kunga Stream is a modern media streaming platform that provides authentication, media management, and API services. The platform is built using a microservices architecture with Rust as the primary language for all services.

## Architecture

The platform consists of three main services:

1. **Auth Service** (`services/auth`) - Handles user authentication, registration, and session management
2. **API Service** (`services/api`) - Provides the main API endpoints for user and media management
3. **Media Service** (`services/media`) - Manages media ingestion, processing, and storage

### Shared Components

- **Common Library** (`libs/common`) - Shared utilities and database connection logic
- **Database** - PostgreSQL for persistent storage
- **Cache** - Redis for session management and caching

## Services

### Auth Service

The authentication service handles all user authentication-related functionality:

- User registration and login
- JWT token generation and validation
- Session management with Redis
- OAuth integration (Google, Apple)
- Rate limiting
- Password hashing with Argon2

**Endpoints:**
- `POST /auth/register` - User registration
- `POST /auth/login` - User login
- `POST /auth/refresh` - Token refresh
- `POST /auth/logout` - User logout
- `POST /auth/logout-all` - Logout from all devices
- `POST /auth/oauth/authorize` - OAuth authorization
- `POST /auth/oauth/callback` - OAuth callback
- `GET /health` - Health check
- `GET /health/redis` - Redis health check

### API Service

The API service provides the main application endpoints:

- User management
- Media item management
- Session management
- Protected routes requiring authentication

**Endpoints:**
- `GET /health` - Health check
- `POST /users` - Create user
- `GET /users` - Get all users
- `GET /users/:id` - Get user by ID
- `GET /sessions` - Get user sessions
- `DELETE /sessions/:id` - Delete session
- `GET /media` - Get media items (protected)
- `GET /media/:id` - Get media item by ID (protected)
- `POST /media/refresh` - Refresh media library (protected)
- `GET /protected` - Protected test route (protected)

### Media Service

The media service handles media ingestion and processing:

- S3 bucket polling for new media files
- Metadata extraction using FFmpeg
- Thumbnail generation
- Media item database management

## Database Schema

The platform uses PostgreSQL with the following main tables:

### Users
- `id` - UUID primary key
- `username` - Unique username
- `email` - Unique email
- `password_hash` - Hashed password
- Timestamps for creation and updates

### Media Items
- `id` - UUID primary key
- `type` - Media type (video, audio, image)
- `metadata` - JSONB metadata
- `s3_key` - S3 object key
- `status` - Processing status
- `user_id` - Foreign key to users
- Extended metadata (duration, width, height, codecs, etc.)
- Timestamps for creation and updates

### Sessions
- `id` - UUID primary key
- `user_id` - Foreign key to users
- `token_hash` - Hashed session token
- `expires_at` - Expiration timestamp
- Timestamps for creation and updates

### Roles and User Roles
- `roles` - Role definitions with permissions
- `user_roles` - Junction table for user-role relationships

### Processed S3 Objects (Media Service)
- `id` - UUID primary key
- `s3_key` - S3 object key
- `etag` - S3 object ETag
- Timestamps for processing and creation/updates

## Technology Stack

- **Language**: Rust
- **Web Framework**: Axum
- **Database**: PostgreSQL
- **Cache**: Redis
- **Authentication**: JWT
- **Media Processing**: FFmpeg
- **Cloud Storage**: AWS S3
- **Serialization**: Serde, Serde JSON
- **Database ORM**: SQLx
- **Password Hashing**: Argon2
- **OAuth**: OAuth2 crate
- **AWS SDK**: AWS SDK for Rust
- **Task Scheduling**: Tokio Cron Scheduler

## Getting Started

### Prerequisites

- Rust and Cargo (latest stable version)
- Docker and Docker Compose
- PostgreSQL client
- FFmpeg (for media processing)

### Setup

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd joy-kunga.stream
   ```

2. Start the database and cache services:
   ```bash
   docker-compose up -d
   ```

3. Set up environment variables:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

4. Run database migrations for each service:
   ```bash
   # Auth service migrations
   cd services/auth
   # Run your migration tool here
   
   # Media service migrations
   cd services/media
   # Run your migration tool here
   ```

5. Build and run each service:
   ```bash
   # Auth service
   cd services/auth
   cargo run
   
   # API service
   cd services/api
   cargo run
   
   # Media service
   cd services/media
   cargo run
   ```

### Environment Variables

The following environment variables are required:

- Database connection strings for each service
- Redis connection string
- JWT secret keys
- AWS credentials for S3 access
- OAuth client credentials (if using OAuth)

See `.env.example` for a complete list of required environment variables.

## Development

### Project Structure

```
joy-kunga.stream/
├── libs/
│   └── common/              # Shared library
├── services/
│   ├── api/                 # API service
│   ├── auth/                # Authentication service
│   └── media/               # Media processing service
├── migrations/              # Database migrations
├── docker-compose.yml       # Development infrastructure
├── Cargo.toml               # Workspace configuration
└── .env.example            # Environment variable examples
```

### Building

To build all services:
```bash
cargo build
```

To build a specific service:
```bash
cd services/<service-name>
cargo build
```

### Testing

To run tests for all services:
```bash
cargo test
```

To run tests for a specific service:
```bash
cd services/<service-name>
cargo test
```

## Deployment

The services are designed to be deployed independently:

1. Deploy the database and cache infrastructure
2. Configure environment variables for each service
3. Deploy each service to its respective environment
4. Set up load balancing and reverse proxy as needed

## Security

- Passwords are hashed using Argon2
- JWT tokens are used for authentication
- Sessions are managed with Redis
- Rate limiting is implemented for authentication endpoints
- OAuth 2.0 is supported for external authentication

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Write tests if applicable
5. Commit your changes
6. Push to the branch
7. Create a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Author

Stephane SEGNING LAMBOU <stephane@ssegning.com>