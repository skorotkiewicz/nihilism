# Nihilism

> *"Despite everything, it's still you."*

A **2D time loop game** built with a Rust server and React frontend, featuring **infinite narrative paths** powered by an **OpenAI-compatible LLM API**.

## Concept

Inspired by the philosophical depth of games like **Undertale**, **Doki Doki Literature Club**, and **The Map of Tiny Perfect Things**, Nihilism explores:

- **Time loops** that reveal who you truly are when there are no consequences
- **Nihilism vs. Meaning** - the struggle to find beauty in repetition
- **Persistent memory** - like Flowey, the game remembers everything across loops
- **Infinite paths** - LLM-driven narrative branches that adapt to your choices
- **The Mirror Test** - *"Despite everything, it's still you"*

## Architecture

### Server (Rust)
- **Axum** web framework with WebSocket support
- **OpenAI-compatible API** integration for dynamic storytelling
- **Persistent state** management across loops
- Game mechanics: nihilism score, dark/light choices, key memories

### Client (React + Vite)
- Atmospheric dark void aesthetic
- Visual novel-style narrative interface
- **Character portraits** that shift with mood
- **Ambient music** that responds to nihilism score
- Memory panel showing persistent state

## Quick Start

### 1. Configure LLM API

Set environment variables for your OpenAI-compatible API:

```bash
export LLM_BASE_URL="http://localhost:8080/v1"  # Your LLM endpoint
export LLM_API_KEY="sk-your-api-key"            # API key
export LLM_MODEL="gpt-4"                        # Model name
```

### 2. Run the Server

```bash
LLM_BASE_URL="http://localhost:8080/v1" ./nihilism
```

The server starts on port 3001 by default.

### 3. Run the Client (Development)

```bash
cd client
bun run dev
```

Open http://localhost:3000 to play.

### 4. Build for Production

```bash
# Build server
cargo build --release

# Build client
cd client && bun run build
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/health` | GET | Health check |
| `/api/game/new` | POST | Create new game session |
| `/api/game/{id}` | GET | Get game state |
| `/api/game/{id}/start` | POST | Start/continue narrative |
| `/api/game/{id}/choice` | POST | Make a choice |
| `/api/game/{id}/reset` | POST | Reset the loop |
| `/api/game/save/{id}` | POST | Save game to disk |
| `/api/game/load/{id}` | GET | Load game from disk |
| `/api/game/list` | GET | List all saved games |
| `/api/game/{id}/ending` | GET | Check for ending |

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Server bind address |
| `PORT` | `3001` | Server port |
| `LLM_BASE_URL` | `http://localhost:8080/v1` | LLM API base URL |
| `LLM_API_KEY` | `sk-none` | LLM API key |
| `LLM_MODEL` | `gpt-4` | LLM model name |

## Game Mechanics

### Nihilism Score
- Ranges from -100 (hopeful) to +100 (nihilistic)
- Dark choices increase the score
- Light choices decrease it
- Affects narrative tone and available paths

### Persistent Memory
Like Flowey in Undertale, the game remembers:
- Total loops completed
- All choices made (dark vs. light)
- Key narrative moments
- Character deaths
- Truths discovered

### The Loop
Each loop can be reset manually or triggered by narrative events. The world forgets, but the narrator remembers everything.

### Multiple Endings
Reach one of 7 unique endings based on your cumulative choices:

| Ending | Condition |
|--------|-----------|
| **Void Embrace** | High nihilism score, 30+ dark choices |
| **Tiny Perfect Things** | Found meaning despite darkness (-60 score, 25+ light) |
| **Just You** | 15+ loops, 50+ choices, balanced score |
| **Transcendence** | Broke free through positive choices (-80 score) |
| **The Watcher** | Observed many loops without commitment |
| **Acceptance** | Moderate everything across 25+ loops |
| **The Middle Path** | Perfect balance of dark and light (rare) |

### Save System
Games auto-save every 3 choices and on loop reset. Files stored in `data/players/`.

## Themes from the Source Material

> *"Time loops test your characters to their limits, revealing who they truly are by giving them infinite time and no consequences."*

> *"It's easy to do the good thing when you're getting rewarded for it. But when you have every opportunity to do the bad thing, every reason to, and no reason to do the good thing, and nobody would even know anyway—then what do you do?"*

> *"The horror of that meaningless looping existence drives her to sadism, to cruelty, to doing anything and everything."*

> *"Sometimes we can all feel like we're stuck living the same day on repeat. And in that, sometimes we miss the moments of daily magic happening all around us."*

---

*A philosophical time loop experience*
