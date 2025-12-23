# Limitless Backend

A Rust backend for the Limitless browser-based RPG game, built with Axum.

## 🚀 Quick Start

### Prerequisites

- Rust (latest stable version)
- Cargo

### Running the Server

```bash
# Navigate to backend directory
cd backend

# Run in development mode
cargo run

# Or run with logging
RUST_LOG=debug cargo run
```

The server will start on `http://localhost:8080`

## 🏗️ Project Structure

```
src/
├── main.rs              # Server entry point and configuration
├── routes.rs            # Route definitions
├── models.rs            # Data models and DTOs
├── auth.rs              # Authentication utilities (JWT)
├── handlers/            # Request handlers
│   ├── mod.rs
│   ├── auth.rs          # Authentication endpoints
│   ├── player.rs        # Player management endpoints
│   ├── adventure.rs     # Adventure system endpoints
│   └── location.rs      # Game world endpoints
```

## 🔧 Current Status

### ✅ Implemented

- **Web Server**: Axum-based HTTP server
- **CORS Support**: Configured for React frontend
- **Mock Authentication**: Demo login/register
- **Player Management**: Basic player data endpoints
- **Game World**: Location and adventure endpoints
- **Request/Response Models**: Type-safe data structures
- **Logging**: Structured logging with tracing

### 🔄 Mock Data (No Database Yet)

- All endpoints return mock data
- Authentication accepts demo/demo credentials
- Player data is hardcoded
- Adventures and locations are predefined

### 📋 TODO (Database Integration)

- [ ] Database schema design
- [ ] User registration and authentication
- [ ] Player data persistence
- [ ] Adventure system logic
- [ ] Inventory and equipment management
- [ ] Real JWT token generation/validation

## 📡 API Endpoints

### Authentication

- `POST /api/auth/login` - User login
- `POST /api/auth/register` - User registration
- `POST /api/auth/logout` - User logout

### Player Management

- `GET /api/player` - Get player data
- `PATCH /api/player` - Update player data
- `POST /api/inventory/use` - Use item
- `POST /api/inventory/equip` - Equip item
- `POST /api/inventory/unequip` - Unequip item

### Game World

- `GET /api/locations` - List all locations
- `GET /api/locations/:id` - Get specific location
- `POST /api/adventures/:id/start` - Start adventure
- `POST /api/adventures/:id/complete` - Complete adventure

### Health Check

- `GET /api/health` - Server status

## 🎮 Testing with Frontend

1. **Start the backend server**:

   ```bash
   cargo run
   ```

2. **Update frontend to use real backend**:
   In `frontend/src/services/api.ts`, change:

   ```typescript
   const USE_MOCK_API = false;
   ```

3. **Test the integration**:
   - Visit `http://localhost:5173`
   - Login with demo/demo
   - Try adventures and explore locations

## 🛠️ Development

### Adding New Endpoints

1. **Define route** in `src/routes.rs`
2. **Create handler** in appropriate `src/handlers/*.rs` file
3. **Add models** in `src/models.rs` if needed

### Example: Adding a new endpoint

```rust
// In routes.rs
.route("/api/new-endpoint", get(handlers::new_handler))

// In handlers/mod.rs or new file
pub async fn new_handler() -> Json<SomeResponse> {
    // Implementation
}
```

### Running with Different Log Levels

```bash
# Debug level
RUST_LOG=debug cargo run

# Info level
RUST_LOG=info cargo run

# Trace level (very verbose)
RUST_LOG=trace cargo run
```

## 📦 Dependencies

- **axum** - Modern web framework
- **tokio** - Async runtime
- **tower-http** - Middleware (CORS, tracing)
- **serde** - Serialization/deserialization
- **uuid** - Unique identifier generation
- **chrono** - Date/time handling
- **jsonwebtoken** - JWT authentication (TODO)
- **bcrypt** - Password hashing (TODO)
- **tracing** - Structured logging

## 🔄 Next Steps

### Phase 1: Database Integration

1. Add SQLx or Diesel for database operations
2. Create database schema and migrations
3. Replace mock data with real database queries

### Phase 2: Authentication

1. Implement proper JWT token generation
2. Add password hashing with bcrypt
3. Create user registration validation

### Phase 3: Game Logic

1. Implement adventure mechanics
2. Add inventory management
3. Create equipment system
4. Build progression and leveling

### Phase 4: Advanced Features

1. Real-time features with WebSockets
2. Multiplayer functionality
3. Daily rollover mechanics
4. Admin tools and monitoring
