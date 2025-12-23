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
└── backend/              # Rust backend (to be implemented)
    ├── src/
    ├── migrations/
    ├── Cargo.toml
    └── README.md
```

## 🚀 Getting Started

### Frontend Setup (Current)

The React frontend is fully functional with mock data:

```bash
cd frontend
npm install
npm run dev
```

Visit `http://localhost:5173` and log in with:

- Username: `demo`
- Password: `demo`

### Backend Setup (Next Steps)

The Rust backend is designed but not yet implemented. Expected structure:

- **Web Framework:** Axum or Actix-web
- **Database:** PostgreSQL with Diesel ORM
- **Authentication:** JWT tokens
- **API:** RESTful endpoints matching frontend expectations

## 🎮 Game Features

### Implemented (Frontend)

- ✅ User authentication system
- ✅ Player character display with stats
- ✅ Adventure system with location selection
- ✅ Equipment management
- ✅ Experience and leveling
- ✅ Responsive game UI with fantasy theme

### Planned (Backend Required)

- 🔄 Persistent player data
- 🔄 Real adventure mechanics
- 🔄 Item and equipment system
- 🔄 Shop and trading
- 🔄 Multiplayer features
- 🔄 Daily rollover mechanics

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

### Backend (Planned)

- **Rust** - Systems programming language
- **Axum/Actix-web** - Web framework
- **PostgreSQL** - Database
- **Diesel** - ORM and migrations
- **JWT** - Authentication
- **Serde** - Serialization

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

### Phase 2: Backend Foundation 🔄

- [ ] Rust project setup with web framework
- [ ] Database schema design and migrations
- [ ] User authentication with JWT
- [ ] Player data management APIs
- [ ] Adventure system implementation

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

1. **Implement Rust Backend** - Set up the server architecture
2. **Database Schema** - Design tables for players, items, adventures
3. **Authentication System** - Secure JWT-based auth
4. **Adventure Engine** - Logic for processing adventures
5. **Data Migration** - Move from mock data to real persistence

Ready to start building your own Kingdom of Loathing inspired game? Begin with the frontend in the `frontend/` directory!
