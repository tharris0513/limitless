# Limitless - Frontend

A React + TypeScript frontend for a browser-based game inspired by Kingdom of Loathing, built with Vite.

## 🚀 Quick Start

### Prerequisites

- Node.js (v16 or higher)
- npm or yarn

### Installation

1. Clone the repository and navigate to the frontend directory:

```bash
cd frontend
```

2. Install dependencies:

```bash
npm install
```

3. Start the development server:

```bash
npm run dev
```

The application will be available at `http://localhost:5173`

## 🎮 Demo Login

For testing purposes, you can use these demo credentials:

- **Username:** `demo`
- **Password:** `demo`

Or create a new account using any username/email combination.

## 🏗️ Project Structure

```
src/
├── components/          # Reusable UI components
│   └── GameLayout.tsx   # Main game layout with player stats
├── pages/              # Page components
│   ├── LoginPage.tsx   # Authentication page
│   └── GamePage.tsx    # Main game interface
├── services/           # API and data services
│   ├── api.ts          # Backend API communication
│   └── mockApi.ts      # Mock data for development
├── types/              # TypeScript type definitions
│   └── game.ts         # Game-related interfaces
└── App.tsx            # Main application component
```

## 🎯 Features

### Current Features

- ✅ User authentication (login/register)
- ✅ Player stats display (health, mana, experience, adventures)
- ✅ Character stats (muscle, mysticism, moxie)
- ✅ Equipment system
- ✅ Location-based adventures
- ✅ Experience and leveling system
- ✅ Responsive design with game-like styling

### Kingdom of Loathing Inspired Elements

- **Adventure System:** Limited daily adventures with various locations
- **Character Stats:** Classic RPG stats (Muscle, Mysticism, Moxie)
- **Equipment Slots:** Weapon, Armor, and Accessory slots
- **Experience & Leveling:** Traditional RPG progression
- **Text-Based Adventures:** Focus on narrative and choices

## 🛠️ Tech Stack

- **Frontend Framework:** React 19 with TypeScript
- **Build Tool:** Vite
- **Styling:** Styled-components
- **HTTP Client:** Axios
- **Icons:** Lucide React
- **Routing:** React Router DOM (ready to use)

## 🎨 Styling

The game uses a fantasy-themed color scheme:

- **Primary Colors:** Gold (#ffd700) and Blue gradients
- **Background:** Dark blue gradient with transparency effects
- **Cards:** Semi-transparent with golden borders on hover
- **Typography:** Clean, readable fonts with golden accents

## 🔧 Configuration

### Environment Variables

Create a `.env` file in the frontend directory:

```env
# Backend API URL - update when your Rust backend is ready
VITE_API_URL=http://localhost:8080/api

# Game configuration
VITE_GAME_NAME="Limitless"
VITE_GAME_VERSION="1.0.0"
```

### Development vs Production

The app currently uses mock API responses for development. To connect to your Rust backend:

1. Set `USE_MOCK_API = false` in `src/services/api.ts`
2. Update `VITE_API_URL` in your `.env` file
3. Ensure your backend implements the expected API endpoints

## 📡 API Integration

The frontend is designed to work with a Rust backend. Expected API endpoints:

### Authentication

- `POST /api/auth/login` - User login
- `POST /api/auth/register` - User registration
- `POST /api/auth/logout` - User logout

### Player Management

- `GET /api/player` - Get player data
- `PATCH /api/player` - Update player data

### Game World

- `GET /api/locations` - Get all locations
- `GET /api/locations/:id` - Get specific location
- `POST /api/adventures/:id/start` - Start an adventure
- `POST /api/adventures/:id/complete` - Complete an adventure

### Inventory & Equipment

- `POST /api/inventory/use` - Use an item
- `POST /api/inventory/equip` - Equip an item
- `POST /api/inventory/unequip` - Unequip an item

## 🎮 Game Mechanics

### Adventure System

- Players have limited daily adventures (default: 10)
- Adventures are location-based with varying difficulty
- Completing adventures grants experience and rewards
- Adventure costs decrease available adventure count

### Character Progression

- **Level:** Overall character progression
- **Experience:** Points toward next level
- **Health/Mana:** Combat and magic resources
- **Stats:** Muscle (strength), Mysticism (magic), Moxie (dexterity)

### Equipment System

- Three equipment slots: Weapon, Armor, Accessory
- Items have rarity levels and types
- Equipment affects character capabilities

## 🚀 Next Steps

### Immediate Enhancements

1. **Inventory Management:** Full inventory interface with drag-and-drop
2. **Shop System:** Item purchasing and selling
3. **Combat System:** Turn-based combat mechanics
4. **Quest System:** Story-driven quests and objectives

### Advanced Features

1. **Multiplayer Chat:** Player communication system
2. **Guild System:** Player organizations and cooperation
3. **PvP Arena:** Player vs player combat
4. **Crafting System:** Item creation and enhancement
5. **Achievement System:** Goals and rewards

## 🔨 Development Commands

```bash
# Start development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview

# Run linting
npm run lint
```

## 📝 Notes for Rust Backend Integration

When implementing the Rust backend, ensure:

1. **CORS Configuration:** Allow requests from the frontend origin
2. **JWT Authentication:** Implement secure token-based auth
3. **Database Models:** Match the TypeScript interfaces in `types/game.ts`
4. **Error Handling:** Return consistent error responses
5. **Rate Limiting:** Prevent abuse of adventure and API endpoints

## 🎯 Kingdom of Loathing Inspiration

This game draws inspiration from the classic browser game Kingdom of Loathing:

- **Humorous tone** and quirky item descriptions
- **Daily adventure limits** to encourage regular play
- **Stat-based character building** with meaningful choices
- **Text-heavy gameplay** with rich narrative
- **Community features** for social interaction

The goal is to capture the charm and addictive gameplay of KoL while building something unique and modern.
