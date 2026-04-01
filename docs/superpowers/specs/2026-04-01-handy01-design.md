# Handy01 — Real-Time Voice Dictation with AI Contextual Correction

**Date:** 2026-04-01  
**Status:** Approved for implementation  
**Stack:** Tauri 2.x + React 18 + Rust + Python FastAPI sidecar + faster-whisper + hybrid LLM correction

---

## Problem Statement

Standard voice-to-text tools transcribe words literally, producing errors when homophones, technical terms, or context-dependent words are misrecognized. Users must manually edit the output, defeating the purpose of voice dictation.

Handy01 solves this by running AI-powered contextual correction **in real-time** — fixing errors as the user speaks, before sentences are finalized, with configurable correction styles.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri App (React UI)                  │
│  ┌──────────┐  ┌──────────────┐  ┌───────────────────┐  │
│  │ Settings  │  │ Live Text    │  │ Correction Style  │  │
│  │ Panel     │  │ Display      │  │ Selector          │  │
│  └──────────┘  └──────────────┘  └───────────────────┘  │
└──────────────────────────┬──────────────────────────────┘
                           │ Tauri IPC (commands/events)
┌──────────────────────────┴──────────────────────────────┐
│                    Rust Core Layer                        │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────┐  │
│  │ Audio Capture │  │ Hotkey Mgr   │  │ Text Injector  │  │
│  │ (cpal)       │  │ (global)     │  │ (rdev/enigo)   │  │
│  └──────┬───────┘  └──────────────┘  └────────┬───────┘  │
│         │                                      │          │
│         ▼                                      ▲          │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────┐  │
│  │ Stream Buffer │→│ Correction    │→│ Output Router   │  │
│  │ (chunked WAV) │  │ Orchestrator │  │ (inject/display)│  │
│  └──────┬───────┘  └──────┬───────┘  └────────────────┘  │
│         │                 │                               │
└─────────┼─────────────────┼───────────────────────────────┘
          │                 │
          ▼                 ▼
┌─────────────────────────────────────────────────────────┐
│                  Python Sidecar (FastAPI)                 │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────┐  │
│  │ Whisper       │  │ Local LLM    │  │ Cloud LLM      │  │
│  │ (faster-      │  │ (tiny/       │  │ (OpenAI/       │  │
│  │ whisper)      │  │ small)       │  │  Anthropic)    │  │
│  └──────────────┘  └──────────────┘  └────────────────┘  │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ Hybrid Correction Router:                           │  │
│  │ 1. Local fix common errors (homophones, context)    │  │
│  │ 2. Escalate to cloud when confidence low or complex │  │
│  └─────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Real-Time Correction Pipeline

### Streaming Algorithm

```
User speaks → Audio chunks (500ms, 250ms overlap) → Whisper streaming
                                                   ↓
                                             Raw text segment
                                                   ↓
                               ┌─────────────────────────────┐
                               │   Correction Decision Tree   │
                               │                              │
                               │ 1. Confidence >= 0.90?       │
                               │    → Pass through, no fix    │
                               │                              │
                               │ 2. Confidence 0.70-0.90?     │
                               │    → Local LLM fix           │
                               │    (homophones, grammar)     │
                               │                              │
                               │ 3. Confidence < 0.70?        │
                               │    → Cloud LLM fix           │
                               │    (full context rewrite)    │
                               │                              │
                               │ 4. Sentence boundary hit?    │
                               │    → Finalize, lock text     │
                               └─────────────────────────────┘
                                                   ↓
                                     Correction style applied
                                     (inline/highlight/draft/configurable)
                                                   ↓
                                     Inject into active app
```

### Key Algorithm Details

- **Chunk size:** 500ms overlapping windows (250ms overlap) to avoid cutting words mid-phoneme
- **Buffer management:** Sliding window keeps last 3-5 seconds of raw text for context
- **Sentence boundary detection:** VAD silence (>500ms) + terminal punctuation (`.`, `!`, `?`)
- **Correction lock:** Once a sentence is finalized, it's never re-corrected (prevents flickering)
- **Cloud fallback timeout:** 800ms — if cloud LLM doesn't respond, fall back to local or pass through
- **Deduplication:** Overlapping chunks produce repeated text; merge algorithm detects and removes duplicates

### Hybrid Model Selection

| Tier | Model | Size | VRAM | Latency | Use Case |
|------|-------|------|------|---------|----------|
| Local | Phi-3-mini / TinyLlama | ~1-2GB | 1-2GB | <50ms | Homophone fixes, grammar, common errors |
| Cloud | GPT-4o-mini / Claude Haiku | N/A | N/A | 200-800ms | Complex context, domain terms, low confidence |
| Fallback | Rule-based dictionary | <1MB | 0 | <1ms | No model available, common word substitutions |

## Data Flow

1. **User presses hotkey** (default: `Ctrl+Shift+Space`) → Rust activates mic, sends "recording" event to React UI
2. **Audio streaming loop** (every 500ms): cpal captures PCM → resample to 16kHz mono → encode WAV → POST to Python sidecar `/transcribe`
3. **Python sidecar processes chunk:** faster-whisper transcribes → returns `{text, confidence, ts}` → Correction router evaluates (local vs cloud) → returns `{corrected_text, style, locked}`
4. **Rust receives correction result:** Sliding window merges with previous text → applies correction style → TextInjector simulates keystrokes into focused app → sends display update to React UI
5. **Sentence boundary detected:** Lock text, clear buffer, reset context window → React UI shows "finalized" indicator
6. **User presses hotkey again:** Stop recording → flush remaining buffer, cleanup

## Component Breakdown

### Rust Core (`src/`)

| Module | Responsibility | Key Crates |
|--------|---------------|------------|
| `audio/` | Mic capture, resampling, chunking | `cpal`, `hound`, `rubato` |
| `hotkey/` | Global hotkey registration | `tauri::global_shortcut` |
| `inject/` | Text injection into active window | `rdev`, `enigo` |
| `sidecar/` | Python process management, HTTP client | `reqwest`, `tokio`, `which` |
| `correction/` | Sliding window, merge logic, style application | (pure Rust) |
| `state/` | Shared app state, settings persistence | `tauri::State`, `serde` |

### Python Sidecar (`sidecar/`)

| Module | Responsibility | Key Libraries |
|--------|---------------|---------------|
| `transcribe.py` | Whisper model loading, chunk processing | `faster-whisper` |
| `correction/` | Hybrid router, local LLM, cloud client | `transformers`, `openai`, `anthropic` |
| `server.py` | FastAPI endpoints, async processing | `fastapi`, `uvicorn` |
| `models/` | Model management, download, caching | `huggingface_hub` |

### React UI (`src/`)

| Component | Responsibility |
|-----------|---------------|
| `App` | Main layout, routing |
| `LiveTranscript` | Real-time text display with correction style rendering |
| `SettingsPanel` | Hotkey config, model selection, correction style, API keys |
| `StatusIndicator` | Recording state, latency, model status, connection health |
| `CorrectionPreview` | Interactive demo of each correction style |

## Error Handling

| Scenario | Detection | Recovery |
|----------|-----------|----------|
| Python sidecar not running | Health check ping on startup | Auto-start sidecar, or show install prompt |
| Sidecar crashes mid-session | HTTP timeout / connection refused | Auto-restart, warn user, preserve audio buffer |
| Whisper model not downloaded | FileNotFoundError on model load | Download on first run with progress bar |
| Cloud API rate limited / down | HTTP 429 or timeout | Fall back to local-only mode, notify user |
| Local LLM OOM | CUDA OOM error | Downgrade to smaller model or disable local correction |
| No mic permission | cpal returns error | OS-specific permission prompt, graceful degrade |
| Text injection fails | rdev/enigo returns error | Fallback to clipboard paste (Ctrl+V) |
| Network drops (cloud mode) | Connection timeout | Seamless fallback to local, auto-reconnect |

## Correction Styles

| Style | Behavior | Best For |
|-------|----------|----------|
| **Inline** | Wrong words silently fix themselves | Fast dictation, trusted users |
| **Highlighted** | Corrected words shown with subtle underline/highlight | Users who want visibility |
| **Draft → Final** | Raw text appears, polished version fades in after 1-2s | Maximum transparency |
| **Configurable** | User chooses per-session or per-app | Power users |

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| End-to-end latency | < 200ms | Hotkey press to first text appearing |
| Per-chunk transcription | < 100ms | Audio chunk to raw text |
| Local correction | < 50ms | Raw text to corrected text |
| Cloud correction | < 800ms | Raw text to corrected text (with timeout) |
| Memory usage | < 2GB total | Rust + Python + models |
| Binary size | < 100MB | Tauri bundle |

## Testing Strategy

| Layer | Tool | What |
|-------|------|------|
| Rust unit tests | `cargo test` | Audio chunking, correction merge logic, sliding window, deduplication |
| Rust integration | `cargo test --test` | Sidecar communication, text injection mocks |
| Python unit | `pytest` | Correction router decisions, model loading, confidence thresholds |
| Python integration | `pytest + httpx` | FastAPI endpoints, mock Whisper responses |
| E2E | Playwright | Full recording → transcription → injection flow |
| Performance | Custom benchmark | End-to-end latency < 200ms target |

## Project Structure

```
handy01/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── audio/
│   │   │   ├── mod.rs
│   │   │   ├── capture.rs
│   │   │   ├── resample.rs
│   │   │   └── chunker.rs
│   │   ├── hotkey/
│   │   │   ├── mod.rs
│   │   │   └── manager.rs
│   │   ├── inject/
│   │   │   ├── mod.rs
│   │   │   └── text_injector.rs
│   │   ├── sidecar/
│   │   │   ├── mod.rs
│   │   │   ├── client.rs
│   │   │   └── process.rs
│   │   ├── correction/
│   │   │   ├── mod.rs
│   │   │   ├── sliding_window.rs
│   │   │   ├── merge.rs
│   │   │   └── style.rs
│   │   └── state/
│   │       ├── mod.rs
│   │       ├── settings.rs
│   │       └── app_state.rs
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── build.rs
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── components/
│   │   ├── LiveTranscript.tsx
│   │   ├── SettingsPanel.tsx
│   │   ├── StatusIndicator.tsx
│   │   └── CorrectionPreview.tsx
│   └── styles/
├── sidecar/
│   ├── server.py
│   ├── transcribe.py
│   ├── correction/
│   │   ├── __init__.py
│   │   ├── router.py
│   │   ├── local_llm.py
│   │   └── cloud_llm.py
│   ├── models/
│   │   ├── __init__.py
│   │   └── manager.py
│   ├── requirements.txt
│   └── pyproject.toml
├── tests/
│   ├── e2e/
│   └── performance/
├── docs/
│   └── superpowers/specs/
│       └── 2026-04-01-handy01-design.md
├── package.json
├── vite.config.ts
├── tsconfig.json
└── README.md
```

## Implementation Phases

### Phase 1: Foundation (Week 1)
- Tauri project scaffold
- Python sidecar with Whisper transcription
- Basic Rust audio capture and chunking
- HTTP communication between Rust and Python

### Phase 2: Correction Engine (Week 2)
- Hybrid correction router
- Local LLM integration
- Cloud LLM integration
- Sliding window and merge logic

### Phase 3: Text Injection (Week 3)
- Cross-platform text injection
- Hotkey management
- Correction style rendering

### Phase 4: UI Polish (Week 4)
- React UI components
- Settings persistence
- Status indicators and health checks

### Phase 5: Testing & Performance (Week 5)
- Unit and integration tests
- E2E test suite
- Performance benchmarking
- Optimization
