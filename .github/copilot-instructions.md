# Limitless - AI Agent Guidelines

## Architecture Overview

This is a full-stack browser RPG game inspired by Kingdom of Loathing with a **Rust + Axum backend** and **React + TypeScript frontend**. The architecture follows a clean separation:

- **Backend** (`backend/`): Rust/Axum REST API with AWS DynamoDB for persistence
- **Frontend** (`frontend/`): React 19 + Vite SPA with TypeScript, CSS Modules
- **Authentication**: Discord OAuth2 → JWT tokens (24hr expiry) stored in localStorage
- **Data Flow**: Frontend contexts → API service → Backend handlers → Repository layer → DynamoDB

## Critical Patterns & Conventions

### Backend (Rust)

**Repository Pattern**: All DynamoDB access goes through `repository/` modules - NEVER call `client` directly from handlers:

```rust
// ✅ Correct: use repository methods
repo.get_user_by_discord_id(&discord_id).await?

// ❌ Wrong: direct DynamoDB calls in handlers
client.get_item().table_name(table).send().await?
```

**Error Handling**: Use `AppError` wrapper (defined in `error.rs`) for all HTTP errors:

```rust
use crate::error::AppError;

// Returns proper HTTP status codes with JSON error responses
Err(AppError::not_found("Character not found"))
```

**Auth Middleware**: Two extractors pattern for protected routes:

- `AuthClaims(claims)` - JWT-authenticated users
- `AdminClaims(claims)` - Admin-only routes (auto-rejects non-admins with 403)

**Damage Formulas**: Use `evalexpr` crate for safe formula evaluation. Formulas stored as strings in DB:

```rust
// Abilities have damage_formula, heal_formula, effect_formula fields
damage_calculator::calculate_damage(ability, caster, target)?
```

See [`DAMAGE_FORMULAS.md`](../DAMAGE_FORMULAS.md) for variable names and examples. All damage auto-includes ±10% variation unless `rand()` used.

### Frontend (TypeScript)

**API Service Pattern**: All backend calls go through `services/api.ts` - NEVER use axios directly:

```typescript
// ✅ Correct
import { GameAPI } from "../services/api";
const user = await GameAPI.getUser();

// ❌ Wrong
axios.get("/api/user");
```

**State Management**:

- **Global**: `GameStateContext` for current game state (combat, exploration) - auto-saves to `Character.game_state` JSON field
- **Loading**: `LoadingContext` for async operations tracking
- Use custom hooks: `useGameState()`, `useLoading()`, `useApiWithLoading()`
- **NEVER LET THE FRONT END MODIFY STATE**. The front end only tells the backend what to do; the backend is the source of truth.
- The backend must validate all state changes.

**Naming Convention**: Backend uses `snake_case`, frontend uses `camelCase`. Transform in `api.ts`:

```typescript
function transformUserFromBackend(backendUser: any): User {
  return {
    discordId: backendUser.discord_id,
    createdAt: backendUser.created_at,
    // ...
  };
}
```

## Developer Workflows

### Running Locally

```bash
# Terminal 1 - Backend
cd backend
cargo run                    # Dev server on :8080

# Terminal 2 - Frontend
cd frontend
npm run dev                  # Vite dev server on :5173
```

### Testing Backend

```bash
cd backend
cargo test                   # Run unit tests (mainly in damage_calculator.rs)
cargo check                  # Fast compile check without building
```

### Environment Setup

Both backend and frontend need `.env` files (copy from `.env.example`):

- **Backend**: AWS credentials, Discord OAuth (client ID/secret), `JWT_SECRET`
- **Frontend**: `VITE_API_URL=http://localhost:8080/api`

### Database Seeding

Initial game config (classes, abilities) lives in [`backend/seeds/game-config.json`](../backend/seeds/game-config.json). Import via admin endpoints (not automated yet).

## Key Integration Points

### Discord OAuth Flow

1. Frontend → `GET /api/auth/discord` → redirect URL + state token
2. User authorizes on Discord
3. Discord → callback with code
4. Frontend → `GET /api/auth/discord/callback?code=...` → JWT token + user data
5. Frontend stores JWT in localStorage, sets Authorization header

### Character Game State

Game state (combat progress, exploration position) persisted as JSON in `Character.game_state`:

```typescript
// Auto-saved by GameStateContext, debounced
const { gameState, saveGameState } = useGameState();
```

### WebSocket Chat

Separate router with shared state: `Router::new().route("/chat/ws", ...).with_state(chat_state)` - only chat endpoints use this state.

## Common Gotchas

1. **CORS**: Backend must trust `FRONTEND_URL` env var. Credentials enabled (`withCredentials: true`)
2. **JWT Extractors**: Use `AuthClaims` or `AdminClaims` in handler signatures - middleware populates them
3. **DynamoDB Schema**: Single-table design, `PK` = entity type prefix (e.g., `USER#<id>`), `SK` for relationships
4. **Stat Calculations**: Character stats live in separate `stats` object (not top-level fields)
5. **Route Layering**: Public routes first, then protected (with JWT middleware), then admin routes in `routes.rs`
6. **Error Serialization**: AppError auto-converts to JSON responses via IntoResponse impl

## File Organization Signals

- **Handlers** (`backend/src/handlers/`): One file per resource (user, character, admin, etc.)
- **Repository** (`backend/src/repository/`): Mirrors handler structure, contains all DB queries
- **Pages** (`frontend/src/pages/`): Full-page components with `.module.css` for styling
- **Components** (`frontend/src/components/`): Reusable UI pieces (also with CSS Modules)
- **Types** (`frontend/src/types/`): `game.ts` for API models, `gameState.ts` for frontend state

## When Editing...

- **Adding API Endpoint**: Handler → route in `routes.rs` → API service method → TypeScript types
- **New Game Mechanic**: Update `DAMAGE_FORMULAS.md` if formula-based, add to `game-config.json` for seeding
- **Protected Route**: Add to `protected_routes` in `routes.rs` (JWT middleware auto-applies)
- **Admin Feature**: Use `AdminClaims(claims)` extractor - automatic 403 for non-admins
