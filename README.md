# Limitless

A browser-based RPG game inspired by Kingdom of Loathing, built with a modern React + TypeScript frontend and designed for a Rust backend.

## 🏰 Project Overview

This project aims to recreate the charm and addictive gameplay of Kingdom of Loathing with modern web technologies. The game features:

- **Text-based adventures** with rich narrative elements
- **Character progression** with classic RPG stats (Muscle, Mysticism, Moxie)
- **Equipment system** with weapons, armor, and accessories
- **Daily adventure limits** encouraging regular engagement
- **Location-based exploration** with multiple areas to discover
- **Social features** for player interaction (planned)

## 📁 Project Structure

```
limitless/
├── frontend/           # React + TypeScript frontend (Vite)
│   ├── src/
│   │   ├── components/    # Reusable UI components
│   │   ├── pages/        # Page components
│   │   ├── services/     # API and data services
│   │   ├── types/        # TypeScript definitions
│   │   └── ...
│   └── README.md         # Frontend-specific documentation
└── backend/              # Rust backend (✅ Complete)
    ├── src/
    │   ├── handlers/      # API endpoint handlers
    │   ├── models.rs      # Data models
    │   ├── repository.rs  # DynamoDB data access
    │   └── ...
    ├── Cargo.toml
    └── README.md
```

## 🚀 Getting Started

### Prerequisites

- **Frontend**: Node.js 18+, npm
- **Backend**: Rust (latest stable), AWS CLI with DynamoDB access
- **Auth**: Discord OAuth application

### Quick Setup

See [INTEGRATION.md](./INTEGRATION.md) for complete setup instructions.

**TL;DR:**

1. Set up environment variables (`.env` files in both directories)
2. Start backend: `cd backend && cargo run`
3. Start frontend: `cd frontend && npm install && npm run dev`
4. Visit `http://localhost:5173`

### Frontend Setup

The React frontend is fully functional:

```bash
cd frontend
cp .env.example .env  # Configure VITE_API_URL
npm install
npm run dev
```

Visit `http://localhost:5173` and log in with Discord OAuth.

### Backend Setup

The Rust backend is complete and ready to run:

```bash
cd backend
cp .env.example .env  # Configure AWS, Discord OAuth, JWT secret
cargo run
```

Requires AWS credentials configured for DynamoDB access.

**Features:**

- **Web Framework:** Axum with async/await
- **Database:** AWS DynamoDB
- **Authentication:** JWT tokens with Discord OAuth
- **API:** RESTful endpoints for all game features

## 🎮 Game Features

### Implemented

- ✅ User authentication with Discord OAuth
- ✅ Player character creation and management
- ✅ Persistent player data in DynamoDB
- ✅ Adventure system with location-based gameplay
- ✅ Equipment and inventory management
- ✅ Experience and leveling system
- ✅ Real-time stat calculations
- ✅ Daily adventure limit tracking
- ✅ Admin panel for user management
- ✅ Responsive game UI with fantasy theme

### Future Enhancements

- 📋 Guild system
- 📋 PvP arena
- 📋 Crafting and recipes
- 📋 Achievement system
- 📋 Chat and messaging

## 🛠️ Technology Stack

### Frontend

- **React 19** - UI framework
- **TypeScript** - Type safety
- **Vite** - Build tool and dev server
- **Styled-components** - CSS-in-JS styling
- **Axios** - HTTP client
- **Lucide React** - Icons

### Backend

- **Rust** - Systems programming language
- **Axum** - Async web framework
- **AWS DynamoDB** - NoSQL database
- **AWS SDK** - DynamoDB client
- **JWT** - Token-based authentication
- **Serde** - JSON serialization
- **Discord OAuth** - Third-party authentication

## 🎯 Kingdom of Loathing Inspiration

This project draws heavily from KoL's design philosophy:

### Core Mechanics

- **Limited daily activities** (adventures) encourage regular play
- **Stat-based progression** with meaningful character building choices
- **Humorous tone** with quirky items and descriptions
- **Community focus** with social features and cooperation

### Game Systems

- **Three primary stats** (Muscle/Mysticism/Moxie) affect different playstyles
- **Equipment slots** provide character customization
- **Location-based adventures** with varying difficulty and rewards
- **Item rarity system** creates collecting and trading opportunities

## 🔧 Development Roadmap

### Phase 1: Frontend Foundation ✅

- [x] Project setup with Vite + React + TypeScript
- [x] Basic authentication UI
- [x] Game layout and player stats display
- [x] Adventure interface with location selection
- [x] Mock data for development and testing

### Phase 2: Backend Foundation ✅

- [x] Rust project setup with Axum framework
- [x] DynamoDB schema design and implementation
- [x] User authentication with JWT and Discord OAuth
- [x] Player data management APIs
- [x] Adventure system implementation
- [x] Character creation and selection
- [x] Equipment and inventory system
- [x] Admin endpoints for user management

### Phase 3: Core Gameplay 📋

- [ ] Real adventure mechanics with outcomes
- [ ] Inventory and equipment persistence
- [ ] Shop system with item trading
- [ ] Experience calculation and leveling
- [ ] Daily rollover mechanics

### Phase 4: Social Features 📋

- [ ] Player-to-player messaging
- [ ] Guild creation and management
- [ ] Multiplayer adventures or raids
- [ ] Leaderboards and achievements

## 📝 API Design

The frontend expects the following REST API endpoints:

### Authentication

- `POST /api/auth/register` - Create new account
- `POST /api/auth/login` - User login
- `POST /api/auth/logout` - User logout

### Player Management

- `GET /api/player` - Get player data
- `PATCH /api/player` - Update player data

### Game World

- `GET /api/locations` - List all locations
- `GET /api/locations/:id` - Get location details
- `POST /api/adventures/:id/start` - Begin adventure
- `POST /api/adventures/:id/complete` - Finish adventure

### Inventory

- `POST /api/inventory/use` - Use consumable item
- `POST /api/inventory/equip` - Equip item
- `POST /api/inventory/unequip` - Remove equipped item

See `frontend/src/types/game.ts` for TypeScript interface definitions.

## 🎨 Design Philosophy

### User Experience

- **Immediate engagement** - Players can start adventuring right away
- **Clear progression** - Stats and equipment provide obvious advancement
- **Daily rhythm** - Limited adventures create habitual play patterns
- **Social interaction** - Community features encourage long-term engagement

### Technical Approach

- **Type safety** - TypeScript interfaces shared between frontend and backend
- **Modern tooling** - Vite for fast development, Rust for performance
- **Responsive design** - Works well on desktop and mobile devices
- **Progressive enhancement** - Core features work, advanced features enhance

## 📄 License

This project is open source. Please respect the original Kingdom of Loathing's intellectual property while drawing inspiration from its design.

## 🤝 Contributing

Contributions welcome! Areas where help is needed:

1. **Backend Implementation** - Rust web server and database
2. **Game Balance** - Adventure difficulty and reward tuning
3. **UI/UX Polish** - Improved styling and user experience
4. **Content Creation** - New locations, adventures, and items
5. **Testing** - Automated tests for both frontend and backend

## 🎯 Next Steps

1. **Frontend Integration** - Connect frontend to live backend API
2. **Content Expansion** - Add more locations, adventures, and items
3. **Game Balance** - Tune experience rates and adventure difficulty
4. **Enhanced Features** - Implement guilds, PvP, and crafting systems
5. **Deployment** - Deploy to production with proper CI/CD

The full stack is now complete! Both frontend and backend are ready for integration and expansion.
