# Nihilism API Reference

This document provides a detailed reference for the Nihilism game server API and its configuration.

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

### Request/Response Examples

#### Start New Game
`POST /api/game/new`

Returns the newly created player state.

#### Make a Choice
`POST /api/game/{id}/choice`

Request body:
```json
{
  "choice_id": "string",
  "choice_text": "string"
}
```

## Configuration

The server can be configured using environment variables.

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Server bind address |
| `PORT` | `3001` | Server port |
| `LLM_BASE_URL` | `http://localhost:8080/v1` | LLM API base URL |
| `LLM_API_KEY` | `sk-none` | LLM API key |
| `LLM_MODEL` | `gpt-4` | LLM model name |

---

*A philosophical time loop experience*
