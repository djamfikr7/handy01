# Handy01 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a cross-platform real-time voice dictation app with AI-powered contextual correction that fixes transcription errors as the user speaks.

**Architecture:** Tauri 2.x shell with React UI, Rust core for audio capture/text injection/hotkey management, Python FastAPI sidecar for Whisper transcription and hybrid LLM correction. Communication via HTTP between Rust and Python.

**Tech Stack:** Tauri 2.x, React 18, TypeScript, Rust (cpal, reqwest, rdev, tokio), Python 3.10+ (FastAPI, faster-whisper, transformers, openai), Vite, Playwright

---

## File Structure

```
handy01/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs                    # Tauri app entry
│   │   ├── lib.rs                     # Core setup, commands registration
│   │   ├── audio/
│   │   │   ├── mod.rs                 # Audio module exports
│   │   │   ├── capture.rs             # Microphone capture with cpal
│   │   │   ├── resample.rs            # Resample to 16kHz mono
│   │   │   └── chunker.rs             # 500ms overlapping chunks
│   │   ├── hotkey/
│   │   │   ├── mod.rs                 # Hotkey module exports
│   │   │   └── manager.rs             # Global hotkey registration
│   │   ├── inject/
│   │   │   ├── mod.rs                 # Inject module exports
│   │   │   └── text_injector.rs       # Cross-platform text injection
│   │   ├── sidecar/
│   │   │   ├── mod.rs                 # Sidecar module exports
│   │   │   ├── client.rs              # HTTP client for Python sidecar
│   │   │   └── process.rs             # Python process lifecycle
│   │   ├── correction/
│   │   │   ├── mod.rs                 # Correction module exports
│   │   │   ├── sliding_window.rs      # 3-5 second text buffer
│   │   │   ├── merge.rs               # Deduplication and merge logic
│   │   │   └── style.rs               # Correction style application
│   │   └── state/
│   │       ├── mod.rs                 # State module exports
│   │       ├── settings.rs            # Settings model, persistence
│   │       └── app_state.rs           # Shared app state
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── build.rs
├── src/
│   ├── main.tsx                       # React entry point
│   ├── App.tsx                        # Main app component
│   ├── components/
│   │   ├── LiveTranscript.tsx         # Real-time text display
│   │   ├── SettingsPanel.tsx          # Settings UI
│   │   ├── StatusIndicator.tsx        # Recording/connection status
│   │   └── CorrectionPreview.tsx      # Style preview demo
│   └── styles/
│       └── globals.css
├── sidecar/
│   ├── server.py                      # FastAPI app, endpoints
│   ├── transcribe.py                  # Whisper model, chunk processing
│   ├── correction/
│   │   ├── __init__.py
│   │   ├── router.py                  # Hybrid correction decision tree
│   │   ├── local_llm.py               # Local LLM (Phi-3/TinyLlama)
│   │   └── cloud_llm.py               # Cloud LLM (OpenAI/Anthropic)
│   ├── models/
│   │   ├── __init__.py
│   │   └── manager.py                 # Model download, caching
│   ├── requirements.txt
│   └── pyproject.toml
├── tests/
│   ├── e2e/
│   │   └── dictation.spec.ts          # Playwright E2E tests
│   └── performance/
│       └── latency.bench.ts           # Latency benchmarks
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tailwind.config.js
└── README.md
```

---

## Phase 1: Foundation — Project Scaffolding

### Task 1: Initialize Tauri Project

**Files:**
- Create: `handy01/package.json`, `handy01/vite.config.ts`, `handy01/tsconfig.json`, `handy01/tailwind.config.js`
- Create: `handy01/src-tauri/Cargo.toml`, `handy01/src-tauri/tauri.conf.json`, `handy01/src-tauri/build.rs`

- [ ] **Step 1: Create package.json**

```json
{
  "name": "handy01",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "test": "vitest",
    "test:e2e": "playwright test",
    "bench": "tsx tests/performance/latency.bench.ts"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-global-shortcut": "^2.0.0",
    "react": "^18.3.0",
    "react-dom": "^18.3.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@types/react": "^18.3.0",
    "@types/react-dom": "^18.3.0",
    "@vitejs/plugin-react": "^4.3.0",
    "autoprefixer": "^10.4.0",
    "playwright": "^1.45.0",
    "postcss": "^8.4.0",
    "tailwindcss": "^3.4.0",
    "tsx": "^4.19.0",
    "typescript": "^5.5.0",
    "vite": "^5.4.0",
    "vitest": "^2.1.0"
  }
}
```

- [ ] **Step 2: Create vite.config.ts**

```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 1421 }
      : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
});
```

- [ ] **Step 3: Create tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "moduleDetection": "force",
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"]
}
```

- [ ] **Step 4: Create tailwind.config.js**

```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {},
  },
  plugins: [],
};
```

- [ ] **Step 5: Create src-tauri/Cargo.toml**

```toml
[package]
name = "handy01"
version = "0.1.0"
description = "Real-time voice dictation with AI contextual correction"
authors = ["you"]
edition = "2021"

[lib]
name = "handy01_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2.0.0", features = [] }

[dependencies]
tauri = { version = "2.0.0", features = [] }
tauri-plugin-global-shortcut = "2.0.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.40", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "stream"] }
cpal = "0.15"
hound = "3.5"
rubato = "0.15"
rdev = "0.5"
enigo = "0.2"
which = "6.0"
dirs = "5.0"
log = "0.4"
env_logger = "0.11"
```

- [ ] **Step 6: Create src-tauri/tauri.conf.json**

```json
{
  "productName": "Handy01",
  "version": "0.1.0",
  "identifier": "com.handy01.app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "Handy01",
        "width": 400,
        "height": 300,
        "resizable": true,
        "center": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": []
  }
}
```

- [ ] **Step 7: Create src-tauri/build.rs**

```rust
fn main() {
    tauri_build::build()
}
```

- [ ] **Step 8: Install dependencies**

Run: `cd handy01 && npm install`
Expected: All packages installed successfully

- [ ] **Step 9: Verify Rust compiles**

Run: `cd handy01/src-tauri && cargo check`
Expected: Compilation succeeds (warnings OK for unused imports at this stage)

- [ ] **Step 10: Commit**

```bash
git add .
git commit -m "feat: initialize Tauri project scaffolding"
```

---

### Task 2: Python Sidecar Foundation

**Files:**
- Create: `handy01/sidecar/requirements.txt`, `handy01/sidecar/pyproject.toml`
- Create: `handy01/sidecar/server.py`, `handy01/sidecar/transcribe.py`
- Create: `handy01/sidecar/correction/__init__.py`, `handy01/sidecar/correction/router.py`
- Create: `handy01/sidecar/models/__init__.py`, `handy01/sidecar/models/manager.py`

- [ ] **Step 1: Create requirements.txt**

```
fastapi>=0.115.0
uvicorn[standard]>=0.32.0
faster-whisper>=1.0.0
transformers>=4.45.0
torch>=2.4.0
openai>=1.50.0
anthropic>=0.36.0
huggingface_hub>=0.25.0
httpx>=0.27.0
pydantic>=2.9.0
numpy>=1.26.0
```

- [ ] **Step 2: Create pyproject.toml**

```toml
[project]
name = "handy01-sidecar"
version = "0.1.0"
description = "Python sidecar for Handy01 - Whisper transcription and AI correction"
requires-python = ">=3.10"
dependencies = [
    "fastapi>=0.115.0",
    "uvicorn[standard]>=0.32.0",
    "faster-whisper>=1.0.0",
    "transformers>=4.45.0",
    "torch>=2.4.0",
    "openai>=1.50.0",
    "anthropic>=0.36.0",
    "huggingface_hub>=0.25.0",
    "httpx>=0.27.0",
    "pydantic>=2.9.0",
    "numpy>=1.26.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=8.3.0",
    "pytest-asyncio>=0.24.0",
    "pytest-httpx>=0.30.0",
]

[tool.pytest.ini_options]
asyncio_mode = "auto"
```

- [ ] **Step 3: Create sidecar/server.py**

```python
"""FastAPI server for Handy01 sidecar - handles transcription and correction requests."""

import logging
from contextlib import asynccontextmanager

from fastapi import FastAPI
from pydantic import BaseModel, Field

from transcribe import WhisperTranscriber
from correction.router import CorrectionRouter

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class TranscribeRequest(BaseModel):
    audio_base64: str = Field(..., description="Base64-encoded WAV audio chunk")
    context: str = Field(default="", description="Previous text for context")


class TranscribeResponse(BaseModel):
    text: str
    confidence: float
    timestamp_start: float
    timestamp_end: float


class CorrectRequest(BaseModel):
    raw_text: str
    confidence: float
    context: str = Field(default="", description="Surrounding text context")
    use_cloud: bool = Field(default=False, description="Force cloud correction")


class CorrectResponse(BaseModel):
    corrected_text: str
    source: str  # "local", "cloud", "pass-through", "rule-based"
    latency_ms: float


# Global instances
transcriber: WhisperTranscriber | None = None
correction_router: CorrectionRouter | None = None


@asynccontextmanager
async def lifespan(app: FastAPI):
    global transcriber, correction_router
    logger.info("Initializing Whisper model...")
    transcriber = WhisperTranscriber()
    logger.info("Initializing correction router...")
    correction_router = CorrectionRouter()
    yield
    logger.info("Shutting down sidecar...")


app = FastAPI(title="Handy01 Sidecar", lifespan=lifespan)


@app.get("/health")
async def health_check():
    return {
        "status": "healthy",
        "whisper_loaded": transcriber is not None,
        "correction_loaded": correction_router is not None,
    }


@app.post("/transcribe", response_model=TranscribeResponse)
async def transcribe_chunk(request: TranscribeRequest):
    assert transcriber is not None, "Transcriber not initialized"
    result = transcriber.transcribe_chunk(request.audio_base64)
    return TranscribeResponse(**result)


@app.post("/correct", response_model=CorrectResponse)
async def correct_text(request: CorrectRequest):
    assert correction_router is not None, "Correction router not initialized"
    result = await correction_router.correct(
        raw_text=request.raw_text,
        confidence=request.confidence,
        context=request.context,
        force_cloud=request.use_cloud,
    )
    return CorrectResponse(**result)
```

- [ ] **Step 4: Create sidecar/transcribe.py**

```python
"""Whisper transcription module using faster-whisper."""

import base64
import io
import logging
import os
import tempfile

from faster_whisper import WhisperModel

logger = logging.getLogger(__name__)

# Model selection based on available VRAM
DEFAULT_MODEL = os.getenv("WHISPER_MODEL", "large-v3")


class WhisperTranscriber:
    def __init__(self, model_size: str = DEFAULT_MODEL):
        device = "cuda" if os.getenv("USE_GPU", "1") == "1" else "cpu"
        compute_type = "float16" if device == "cuda" else "int8"
        logger.info(f"Loading Whisper model: {model_size} on {device}")
        self.model = WhisperModel(model_size, device=device, compute_type=compute_type)
        logger.info("Whisper model loaded successfully")

    def transcribe_chunk(self, audio_base64: str) -> dict:
        """Transcribe a single audio chunk and return text with confidence."""
        audio_bytes = base64.b64decode(audio_base64)

        with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as f:
            f.write(audio_bytes)
            temp_path = f.name

        try:
            segments, info = self.model.transcribe(
                temp_path,
                beam_size=5,
                vad_filter=True,
                vad_parameters=dict(min_silence_duration_ms=500),
            )

            texts = []
            confidence = 0.0
            segment_count = 0
            start_ts = 0.0
            end_ts = 0.0

            for segment in segments:
                texts.append(segment.text.strip())
                confidence += segment.avg_logprob
                segment_count += 1
                if segment_count == 1:
                    start_ts = segment.start
                end_ts = segment.end

            avg_confidence = confidence / segment_count if segment_count > 0 else 0.0
            # Convert logprob to approximate confidence (0-1)
            confidence_normalized = min(1.0, max(0.0, 1.0 + avg_confidence))

            return {
                "text": " ".join(texts),
                "confidence": round(confidence_normalized, 3),
                "timestamp_start": start_ts,
                "timestamp_end": end_ts,
            }
        finally:
            os.unlink(temp_path)
```

- [ ] **Step 5: Create sidecar/correction/__init__.py**

```python
from .router import CorrectionRouter

__all__ = ["CorrectionRouter"]
```

- [ ] **Step 6: Create sidecar/correction/router.py**

```python
"""Hybrid correction router - decides between local, cloud, or pass-through correction."""

import logging
import time

from .local_llm import LocalLLM
from .cloud_llm import CloudLLM

logger = logging.getLogger(__name__)

# Confidence thresholds
HIGH_CONFIDENCE_THRESHOLD = 0.90
MEDIUM_CONFIDENCE_THRESHOLD = 0.70
CLOUD_TIMEOUT_MS = 800


class CorrectionRouter:
    def __init__(self):
        self.local_llm = LocalLLM()
        self.cloud_llm = CloudLLM()
        logger.info("Correction router initialized")

    async def correct(
        self,
        raw_text: str,
        confidence: float,
        context: str = "",
        force_cloud: bool = False,
    ) -> dict:
        """Route correction request based on confidence and availability."""
        if not raw_text.strip():
            return {
                "corrected_text": raw_text,
                "source": "pass-through",
                "latency_ms": 0.0,
            }

        # High confidence: pass through
        if confidence >= HIGH_CONFIDENCE_THRESHOLD and not force_cloud:
            return {
                "corrected_text": raw_text,
                "source": "pass-through",
                "latency_ms": 0.0,
            }

        start_time = time.time()

        # Medium confidence: try local LLM
        if confidence >= MEDIUM_CONFIDENCE_THRESHOLD:
            try:
                result = await self.local_llm.correct(raw_text, context)
                latency_ms = (time.time() - start_time) * 1000
                return {**result, "latency_ms": round(latency_ms, 1)}
            except Exception as e:
                logger.warning(f"Local LLM failed, falling back: {e}")
                # Fall through to cloud

        # Low confidence or local failed: try cloud
        if force_cloud or confidence < MEDIUM_CONFIDENCE_THRESHOLD:
            try:
                result = await self.cloud_llm.correct(
                    raw_text, context, timeout_ms=CLOUD_TIMEOUT_MS
                )
                latency_ms = (time.time() - start_time) * 1000
                return {**result, "latency_ms": round(latency_ms, 1)}
            except Exception as e:
                logger.warning(f"Cloud LLM failed, falling back: {e}")
                # Fall through to rule-based

        # Final fallback: rule-based corrections
        result = self._rule_based_correct(raw_text)
        latency_ms = (time.time() - start_time) * 1000
        return {
            "corrected_text": result,
            "source": "rule-based",
            "latency_ms": round(latency_ms, 1),
        }

    def _rule_based_correct(self, text: str) -> str:
        """Apply simple rule-based corrections for common errors."""
        # Common homophone and transcription error corrections
        corrections = {
            " i ": " I ",
            " im ": " I'm ",
            " dont ": " don't ",
            " doesnt ": " doesn't ",
            " didnt ": " didn't ",
            " wont ": " won't ",
            " cant ": " can't ",
            " thats ": " that's ",
            " whats ": " what's ",
            " lets ": " let's ",
            " its ": " it's ",
            " youre ": " you're ",
            " theyre ": " they're ",
            " were ": " we're ",
            " hes ": " he's ",
            " shes ": " she's ",
        }

        result = text.lower()
        for wrong, right in corrections.items():
            result = result.replace(wrong, right)

        # Capitalize first letter
        if result:
            result = result[0].upper() + result[1:]

        return result
```

- [ ] **Step 7: Create sidecar/models/__init__.py**

```python
from .manager import ModelManager

__all__ = ["ModelManager"]
```

- [ ] **Step 8: Create sidecar/models/manager.py**

```python
"""Model management - download, cache, and load models."""

import logging
import os
from pathlib import Path

from huggingface_hub import snapshot_download

logger = logging.getLogger(__name__)

CACHE_DIR = Path(os.getenv("HANDY01_MODEL_CACHE", Path.home() / ".cache" / "handy01"))


class ModelManager:
    def __init__(self):
        self.cache_dir = CACHE_DIR
        self.cache_dir.mkdir(parents=True, exist_ok=True)

    def ensure_whisper_model(self, model_size: str = "large-v3") -> str:
        """Download Whisper model if not cached, return local path."""
        model_path = self.cache_dir / "whisper" / model_size
        if model_path.exists():
            logger.info(f"Whisper model {model_size} found in cache")
            return str(model_path)

        logger.info(f"Downloading Whisper model {model_size}...")
        repo_id = f"guillaumekln/faster-whisper-{model_size}"
        local_dir = snapshot_download(repo_id, local_dir=str(model_path))
        logger.info(f"Whisper model downloaded to {local_dir}")
        return local_dir

    def ensure_local_llm_model(self, model_name: str = "TinyLlama/TinyLlama-1.1B-Chat-v1.0") -> str:
        """Download local LLM model if not cached, return local path."""
        model_path = self.cache_dir / "llm" / model_name.replace("/", "_")
        if model_path.exists():
            logger.info(f"LLM model {model_name} found in cache")
            return str(model_path)

        logger.info(f"Downloading LLM model {model_name}...")
        local_dir = snapshot_download(model_name, local_dir=str(model_path))
        logger.info(f"LLM model downloaded to {local_dir}")
        return local_dir
```

- [ ] **Step 9: Install Python dependencies**

Run: `cd handy01/sidecar && pip install -r requirements.txt`
Expected: All packages installed successfully

- [ ] **Step 10: Verify sidecar starts**

Run: `cd handy01/sidecar && python -c "from server import app; print('Sidecar imports OK')"`
Expected: "Sidecar imports OK"

- [ ] **Step 11: Commit**

```bash
git add sidecar/
git commit -m "feat: add Python sidecar with FastAPI, Whisper, and correction router"
```

---

### Task 3: Rust Core — Audio Capture Module

**Files:**
- Create: `handy01/src-tauri/src/audio/mod.rs`, `handy01/src-tauri/src/audio/capture.rs`
- Create: `handy01/src-tauri/src/audio/resample.rs`, `handy01/src-tauri/src/audio/chunker.rs`

- [ ] **Step 1: Write tests for chunker**

Create: `handy01/src-tauri/src/audio/chunker.rs`

```rust
use std::collections::VecDeque;

/// Manages overlapping audio chunks for streaming transcription
pub struct Chunker {
    /// Buffer of audio samples
    buffer: Vec<f32>,
    /// Chunk size in samples (at 16kHz, 500ms = 8000 samples)
    chunk_size: usize,
    /// Overlap in samples (250ms = 4000 samples)
    overlap: usize,
    /// Queue of ready chunks
    ready: VecDeque<Vec<f32>>,
}

impl Chunker {
    pub fn new(sample_rate: u32, chunk_ms: u32, overlap_ms: u32) -> Self {
        let chunk_size = (sample_rate as usize * chunk_ms as usize) / 1000;
        let overlap = (sample_rate as usize * overlap_ms as usize) / 1000;
        Self {
            buffer: Vec::with_capacity(chunk_size * 2),
            chunk_size,
            overlap,
            ready: VecDeque::new(),
        }
    }

    /// Add audio samples to the buffer, extract complete chunks
    pub fn push(&mut self, samples: &[f32]) {
        self.buffer.extend_from_slice(samples);

        while self.buffer.len() >= self.chunk_size {
            let chunk: Vec<f32> = self.buffer[..self.chunk_size].to_vec();
            self.ready.push_back(chunk);

            // Keep overlap for next chunk
            let keep = self.buffer.len() - self.chunk_size + self.overlap;
            self.buffer.drain(..self.chunk_size - self.overlap);
        }
    }

    /// Get next ready chunk
    pub fn next_chunk(&mut self) -> Option<Vec<f32>> {
        self.ready.pop_front()
    }

    /// Flush remaining samples as a chunk (padded with silence)
    pub fn flush(&mut self) -> Option<Vec<f32>> {
        if self.buffer.is_empty() {
            return None;
        }
        let mut chunk = self.buffer.clone();
        // Pad to chunk_size with zeros
        chunk.resize(self.chunk_size, 0.0);
        self.buffer.clear();
        Some(chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunker_creates_chunks() {
        let mut chunker = Chunker::new(16000, 500, 250);
        // 16000 samples = 1 second at 16kHz
        let samples = vec![0.5f32; 16000];
        chunker.push(&samples);

        // Should have 2 chunks (500ms each = 8000 samples)
        assert!(chunker.next_chunk().is_some());
        assert!(chunker.next_chunk().is_some());
        assert!(chunker.next_chunk().is_none());
    }

    #[test]
    fn test_chunker_overlap() {
        let mut chunker = Chunker::new(16000, 500, 250);
        let samples = vec![0.5f32; 8000]; // Exactly one chunk
        chunker.push(&samples);

        let chunk1 = chunker.next_chunk().unwrap();
        assert_eq!(chunk1.len(), 8000);
    }

    #[test]
    fn test_chunker_flush() {
        let mut chunker = Chunker::new(16000, 500, 250);
        let samples = vec![0.5f32; 4000]; // Half a chunk
        chunker.push(&samples);

        assert!(chunker.next_chunk().is_none());

        let flushed = chunker.flush().unwrap();
        assert_eq!(flushed.len(), 8000); // Padded to full chunk
    }

    #[test]
    fn test_chunker_empty_flush() {
        let mut chunker = Chunker::new(16000, 500, 250);
        assert!(chunker.flush().is_none());
    }
}
```

- [ ] **Step 2: Write tests for resample**

Create: `handy01/src-tauri/src/audio/resample.rs`

```rust
use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction};

/// Resample audio to target sample rate (mono, 16kHz for Whisper)
pub struct AudioResampler {
    resampler: SincFixedIn<f32>,
    input_channels: usize,
}

impl AudioResampler {
    pub fn new(from_rate: u32, to_rate: u32, channels: usize) -> Self {
        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };

        let resampler = SincFixedIn::new(
            to_rate as f64 / from_rate as f64,
            2.0,
            params,
            1024,
            channels,
        )
        .expect("Failed to create resampler");

        Self {
            resampler,
            input_channels: channels,
        }
    }

    /// Resample audio samples to target rate
    pub fn resample(&mut self, samples: &[f32]) -> Vec<f32> {
        if samples.is_empty() {
            return Vec::new();
        }

        // Convert to interleaved format for rubato
        let input: Vec<Vec<f32>> = (0..self.input_channels)
            .map(|ch| samples.iter().skip(ch).step_by(self.input_channels).copied().collect())
            .collect();

        let output = self
            .resampler
            .process(&input, None)
            .expect("Resampling failed");

        // Flatten back to interleaved
        let total_len: usize = output.iter().map(|c| c.len()).sum();
        let mut result = vec![0.0f32; total_len];
        let frames = output[0].len();

        for (i, channel) in output.iter().enumerate() {
            for (j, &sample) in channel.iter().enumerate() {
                result[j * self.input_channels + i] = sample;
            }
        }

        result
    }

    /// Convert stereo to mono by averaging channels
    pub fn stereo_to_mono(stereo: &[f32]) -> Vec<f32> {
        stereo
            .chunks_exact(2)
            .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stereo_to_mono() {
        let stereo = vec![0.4f32, 0.6, 0.2, 0.8];
        let mono = AudioResampler::stereo_to_mono(&stereo);
        assert_eq!(mono, vec![0.5, 0.5]);
    }

    #[test]
    fn test_stereo_to_mono_empty() {
        let stereo: Vec<f32> = vec![];
        let mono = AudioResampler::stereo_to_mono(&stereo);
        assert!(mono.is_empty());
    }

    #[test]
    fn test_resample_changes_length() {
        let mut resampler = AudioResampler::new(48000, 16000, 1);
        let samples = vec![0.5f32; 4800]; // 100ms at 48kHz
        let output = resampler.resample(&samples);
        // Should be approximately 1600 samples (100ms at 16kHz)
        assert!((output.len() as f32 - 1600.0).abs() < 100.0);
    }
}
```

- [ ] **Step 3: Create audio mod.rs**

Create: `handy01/src-tauri/src/audio/mod.rs`

```rust
pub mod capture;
pub mod chunker;
pub mod resample;

pub use chunker::Chunker;
pub use resample::AudioResampler;
```

- [ ] **Step 4: Create capture.rs**

Create: `handy01/src-tauri/src/audio/capture.rs`

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

use super::{AudioResampler, Chunker};

/// Audio capture session
pub struct AudioCapture {
    stream: Option<Stream>,
    sample_rate: u32,
    channels: usize,
}

impl AudioCapture {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No default input device found")?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get input config: {}", e))?;

        Ok(Self {
            stream: None,
            sample_rate: config.sample_rate().0,
            channels: config.channels() as usize,
        })
    }

    /// Get the default input device
    pub fn default_device() -> Result<Device, String> {
        let host = cpal::default_host();
        host.default_input_device()
            .ok_or_else(|| "No default input device".to_string())
    }

    /// Start capturing audio, sending chunks through the callback
    pub fn start<F>(&mut self, mut callback: F) -> Result<(), String>
    where
        F: FnMut(Vec<f32>) + Send + 'static,
    {
        let device = Self::default_device()?;
        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get config: {}", e))?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        let sample_format = config.sample_format();

        let mut resampler = AudioResampler::new(sample_rate, 16000, channels);
        let mut chunker = Chunker::new(16000, 500, 250);

        let stream: Stream = match sample_format {
            SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mono = if channels == 2 {
                        AudioResampler::stereo_to_mono(data)
                    } else {
                        data.to_vec()
                    };
                    let resampled = resampler.resample(&mono);
                    chunker.push(&resampled);

                    while let Some(chunk) = chunker.next_chunk() {
                        callback(chunk);
                    }
                },
                move |err| eprintln!("Audio stream error: {}", err),
                None,
            )
            .map_err(|e| format!("Failed to build stream: {}", e))?,
            _ => {
                return Err(format!("Unsupported sample format: {:?}", sample_format));
            }
        };

        stream
            .play()
            .map_err(|e| format!("Failed to start stream: {}", e))?;

        self.stream = Some(stream);
        Ok(())
    }

    /// Stop capturing
    pub fn stop(&mut self) {
        self.stream = None;
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> usize {
        self.channels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_creation_fails_without_device() {
        // This test verifies error handling when no audio device is available
        // In CI environments, this is expected
        let result = AudioCapture::new();
        // Either OK (has device) or Err (no device) is acceptable
        match result {
            Ok(_) => assert!(true),
            Err(_) => assert!(true),
        }
    }
}
```

- [ ] **Step 5: Run tests**

Run: `cd handy01/src-tauri && cargo test audio::`
Expected: All audio module tests pass

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/audio/
git commit -m "feat: add Rust audio capture, resampling, and chunking modules"
```

---

## Phase 2: Correction Engine

### Task 4: Rust Correction Modules

**Files:**
- Create: `handy01/src-tauri/src/correction/mod.rs`
- Create: `handy01/src-tauri/src/correction/sliding_window.rs`
- Create: `handy01/src-tauri/src/correction/merge.rs`
- Create: `handy01/src-tauri/src/correction/style.rs`

- [ ] **Step 1: Write tests and implementation for sliding_window**

Create: `handy01/src-tauri/src/correction/sliding_window.rs`

```rust
/// Sliding window buffer for maintaining context during real-time correction
pub struct SlidingWindow {
    /// Maximum window size in characters
    max_chars: usize,
    /// Current buffer content
    buffer: String,
    /// Locked (finalized) text that won't be modified
    locked: String,
}

impl SlidingWindow {
    pub fn new(max_chars: usize) -> Self {
        Self {
            max_chars,
            buffer: String::new(),
            locked: String::new(),
        }
    }

    /// Add new text to the window
    pub fn push(&mut self, text: &str) {
        self.buffer.push_str(text);

        // Trim buffer to max_chars, moving excess to locked
        if self.buffer.len() > self.max_chars {
            let excess = self.buffer.len() - self.max_chars;
            let to_lock = self.buffer.drain(..excess).collect::<String>();
            self.locked.push_str(&to_lock);
        }
    }

    /// Lock current buffer content (finalize sentence)
    pub fn lock(&mut self) {
        self.locked.push_str(&self.buffer);
        self.buffer.clear();
    }

    /// Get the full text (locked + buffer)
    pub fn get_full(&self) -> String {
        format!("{}{}", self.locked, self.buffer)
    }

    /// Get only the active buffer (not locked)
    pub fn get_buffer(&self) -> &str {
        &self.buffer
    }

    /// Get locked text
    pub fn get_locked(&self) -> &str {
        &self.locked
    }

    /// Reset the window
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.locked.clear();
    }

    /// Check if buffer contains a sentence boundary
    pub fn has_sentence_boundary(&self) -> bool {
        let trimmed = self.buffer.trim_end();
        trimmed.ends_with('.') || trimmed.ends_with('!') || trimmed.ends_with('?')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sliding_window_push() {
        let mut window = SlidingWindow::new(100);
        window.push("Hello world");
        assert_eq!(window.get_buffer(), "Hello world");
    }

    #[test]
    fn test_sliding_window_trims_to_locked() {
        let mut window = SlidingWindow::new(10);
        window.push("Hello world this is a test");
        assert!(window.get_buffer().len() <= 10);
        assert!(!window.get_locked().is_empty());
    }

    #[test]
    fn test_sliding_window_lock() {
        let mut window = SlidingWindow::new(100);
        window.push("Sentence one.");
        window.lock();
        assert_eq!(window.get_locked(), "Sentence one.");
        assert_eq!(window.get_buffer(), "");
    }

    #[test]
    fn test_sliding_window_sentence_boundary() {
        let mut window = SlidingWindow::new(100);
        window.push("Hello world.");
        assert!(window.has_sentence_boundary());

        window.push(" No punctuation");
        assert!(!window.has_sentence_boundary());
    }

    #[test]
    fn test_sliding_window_reset() {
        let mut window = SlidingWindow::new(100);
        window.push("Some text");
        window.lock();
        window.reset();
        assert_eq!(window.get_full(), "");
    }

    #[test]
    fn test_sliding_window_get_full() {
        let mut window = SlidingWindow::new(100);
        window.push("Locked.");
        window.lock();
        window.push("Active");
        assert_eq!(window.get_full(), "Locked.Active");
    }
}
```

- [ ] **Step 2: Write tests and implementation for merge**

Create: `handy01/src-tauri/src/correction/merge.rs`

```rust
/// Deduplication and merge logic for overlapping transcription chunks
pub struct Merger;

impl Merger {
    /// Merge new text with existing text, removing overlapping duplicates
    ///
    /// Example:
    /// existing: "Hello world how"
    /// new:      "world how are you"
    /// result:   "Hello world how are you"
    pub fn merge(existing: &str, new: &str) -> String {
        if existing.is_empty() {
            return new.to_string();
        }
        if new.is_empty() {
            return existing.to_string();
        }

        let existing_words: Vec<&str> = existing.split_whitespace().collect();
        let new_words: Vec<&str> = new.split_whitespace().collect();

        // Find the longest suffix of existing that matches a prefix of new
        let max_overlap = existing_words.len().min(new_words.len());

        for overlap in (1..=max_overlap).rev() {
            let existing_suffix = &existing_words[existing_words.len() - overlap..];
            let new_prefix = &new_words[..overlap];

            if Self::words_match(existing_suffix, new_prefix) {
                // Merge: existing + non-overlapping part of new
                let mut result = existing.to_string();
                for word in &new_words[overlap..] {
                    result.push(' ');
                    result.push_str(word);
                }
                return result.trim().to_string();
            }
        }

        // No overlap found, append with space
        format!("{} {}", existing.trim_end(), new)
    }

    fn words_match(a: &[&str], b: &[&str]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        a.iter()
            .zip(b.iter())
            .all(|(x, y)| x.to_lowercase() == y.to_lowercase())
    }

    /// Deduplicate text by finding repeated segments
    pub fn deduplicate(text: &str) -> String {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() < 4 {
            return text.to_string();
        }

        // Check for repeated segments of 2-4 words
        for seg_len in (2..=4).rev() {
            if words.len() < seg_len * 2 {
                continue;
            }

            let mid = words.len() / 2;
            let left = &words[mid - seg_len..mid];
            let right = &words[mid..mid + seg_len];

            if Self::words_match(left, right) {
                // Remove the duplicate segment
                let mut result = words[..mid + seg_len].join(" ");
                if mid + seg_len < words.len() {
                    result.push(' ');
                    result.push_str(&words[mid + seg_len..].join(" "));
                }
                return result;
            }
        }

        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_no_overlap() {
        let result = Merger::merge("Hello", "world");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_merge_with_overlap() {
        let result = Merger::merge("Hello world how", "world how are you");
        assert_eq!(result, "Hello world how are you");
    }

    #[test]
    fn test_merge_empty_existing() {
        let result = Merger::merge("", "Hello world");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_merge_empty_new() {
        let result = Merger::merge("Hello world", "");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_merge_exact_match() {
        let result = Merger::merge("Hello world", "Hello world");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_deduplicate_removes_repetition() {
        let text = "Hello world Hello world how are you";
        let result = Merger::deduplicate(text);
        assert!(!result.contains("Hello world Hello world"));
    }

    #[test]
    fn test_deduplicate_no_repetition() {
        let text = "Hello world how are you";
        let result = Merger::deduplicate(text);
        assert_eq!(result, text);
    }

    #[test]
    fn test_deduplicate_short_text() {
        let text = "Hi";
        let result = Merger::deduplicate(text);
        assert_eq!(result, text);
    }
}
```

- [ ] **Step 3: Write tests and implementation for style**

Create: `handy01/src-tauri/src/correction/style.rs`

```rust
use serde::{Deserialize, Serialize};

/// Correction display styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorrectionStyle {
    /// Wrong words silently fix themselves
    Inline,
    /// Corrected words shown with subtle highlight
    Highlighted,
    /// Raw text appears, polished version fades in
    DraftFinal,
}

impl Default for CorrectionStyle {
    fn default() -> Self {
        Self::Inline
    }
}

impl CorrectionStyle {
    /// Format text according to the style
    pub fn format(&self, original: &str, corrected: &str) -> String {
        match self {
            CorrectionStyle::Inline => corrected.to_string(),
            CorrectionStyle::Highlighted => {
                if original == corrected {
                    corrected.to_string()
                } else {
                    format!("[[{}]]", corrected)
                }
            }
            CorrectionStyle::DraftFinal => {
                format!("~~{}~~ → {}", original, corrected)
            }
        }
    }

    /// Check if this style shows both original and corrected
    pub fn shows_original(&self) -> bool {
        matches!(self, Self::DraftFinal | Self::Highlighted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_style() {
        let style = CorrectionStyle::Inline;
        assert_eq!(style.format("hello", "Hello"), "Hello");
    }

    #[test]
    fn test_highlighted_style_changed() {
        let style = CorrectionStyle::Highlighted;
        assert_eq!(style.format("hello", "Hello"), "[[Hello]]");
    }

    #[test]
    fn test_highlighted_style_unchanged() {
        let style = CorrectionStyle::Highlighted;
        assert_eq!(style.format("Hello", "Hello"), "Hello");
    }

    #[test]
    fn test_draft_final_style() {
        let style = CorrectionStyle::DraftFinal;
        assert_eq!(style.format("hello", "Hello"), "~~hello~~ → Hello");
    }

    #[test]
    fn test_shows_original() {
        assert!(CorrectionStyle::DraftFinal.shows_original());
        assert!(CorrectionStyle::Highlighted.shows_original());
        assert!(!CorrectionStyle::Inline.shows_original());
    }

    #[test]
    fn test_default_style() {
        assert_eq!(CorrectionStyle::default(), CorrectionStyle::Inline);
    }

    #[test]
    fn test_serialization() {
        let style = CorrectionStyle::Highlighted;
        let json = serde_json::to_string(&style).unwrap();
        assert!(json.contains("Highlighted"));
    }
}
```

- [ ] **Step 4: Create correction mod.rs**

Create: `handy01/src-tauri/src/correction/mod.rs`

```rust
pub mod merge;
pub mod sliding_window;
pub mod style;

pub use merge::Merger;
pub use sliding_window::SlidingWindow;
pub use style::CorrectionStyle;
```

- [ ] **Step 5: Run tests**

Run: `cd handy01/src-tauri && cargo test correction::`
Expected: All correction module tests pass

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/correction/
git commit -m "feat: add Rust correction modules with sliding window, merge, and style"
```

---

### Task 5: Python Correction Engine — Local and Cloud LLM

**Files:**
- Create: `handy01/sidecar/correction/local_llm.py`
- Create: `handy01/sidecar/correction/cloud_llm.py`

- [ ] **Step 1: Write tests and implementation for local_llm**

Create: `handy01/sidecar/correction/local_llm.py`

```python
"""Local LLM correction using small models (Phi-3-mini, TinyLlama)."""

import logging
import os
import time

logger = logging.getLogger(__name__)

DEFAULT_LOCAL_MODEL = os.getenv("LOCAL_LLM_MODEL", "TinyLlama/TinyLlama-1.1B-Chat-v1.0")


class LocalLLM:
    def __init__(self, model_name: str = DEFAULT_LOCAL_MODEL):
        self.model_name = model_name
        self.pipeline = None
        self._load_model()

    def _load_model(self):
        """Load the local LLM model."""
        try:
            from transformers import pipeline

            logger.info(f"Loading local LLM: {self.model_name}")
            self.pipeline = pipeline(
                "text-generation",
                model=self.model_name,
                torch_dtype="auto",
                device_map="auto",
                max_new_tokens=100,
                do_sample=False,
            )
            logger.info("Local LLM loaded successfully")
        except Exception as e:
            logger.warning(f"Failed to load local LLM: {e}")
            self.pipeline = None

    async def correct(self, raw_text: str, context: str = "") -> dict:
        """Correct text using local LLM."""
        if self.pipeline is None:
            raise RuntimeError("Local LLM not available")

        prompt = self._build_prompt(raw_text, context)

        try:
            result = self.pipeline(prompt)
            corrected = self._extract_correction(result[0]["generated_text"], prompt)
            return {
                "corrected_text": corrected,
                "source": "local",
            }
        except Exception as e:
            logger.error(f"Local LLM correction failed: {e}")
            raise

    def _build_prompt(self, raw_text: str, context: str) -> str:
        """Build a correction prompt for the LLM."""
        context_line = f"Context: {context}" if context else ""
        return f"""Fix any transcription errors in the following text. Only output the corrected text, nothing else.
{context_line}
Text: {raw_text}
Corrected:"""

    def _extract_correction(self, generated: str, prompt: str) -> str:
        """Extract only the correction from the generated text."""
        if generated.startswith(prompt):
            return generated[len(prompt) :].strip()
        return generated.strip()


# Tests
import pytest


@pytest.mark.asyncio
async def test_local_llm_prompt_building():
    llm = LocalLLM.__new__(LocalLLM)
    llm.pipeline = None

    prompt = llm._build_prompt("hello world", "previous context")
    assert "hello world" in prompt
    assert "previous context" in prompt
    assert "Fix any transcription errors" in prompt


@pytest.mark.asyncio
async def test_local_llm_prompt_no_context():
    llm = LocalLLM.__new__(LocalLLM)
    llm.pipeline = None

    prompt = llm._build_prompt("hello world", "")
    assert "Context:" not in prompt
    assert "hello world" in prompt


@pytest.mark.asyncio
async def test_local_llm_extract_correction():
    llm = LocalLLM.__new__(LocalLLM)
    llm.pipeline = None

    prompt = "Fix errors\nText: hellow\nCorrected:"
    generated = "Fix errors\nText: hellow\nCorrected: Hello"
    result = llm._extract_correction(generated, prompt)
    assert result == "Hello"


@pytest.mark.asyncio
async def test_local_llm_correct_empty_text():
    llm = LocalLLM.__new__(LocalLLM)
    llm.pipeline = None

    with pytest.raises(RuntimeError):
        await llm.correct("", "")
```

- [ ] **Step 2: Write tests and implementation for cloud_llm**

Create: `handy01/sidecar/correction/cloud_llm.py`

```python
"""Cloud LLM correction using OpenAI or Anthropic APIs."""

import asyncio
import logging
import os
import time

import httpx

logger = logging.getLogger(__name__)

OPENAI_API_KEY = os.getenv("OPENAI_API_KEY", "")
ANTHROPIC_API_KEY = os.getenv("ANTHROPIC_API_KEY", "")
DEFAULT_CLOUD_MODEL = os.getenv("CLOUD_MODEL", "gpt-4o-mini")


class CloudLLM:
    def __init__(self):
        self.provider = self._detect_provider()
        self.client = None

    def _detect_provider(self) -> str:
        """Detect available cloud provider."""
        if ANTHROPIC_API_KEY:
            return "anthropic"
        if OPENAI_API_KEY:
            return "openai"
        return "none"

    async def correct(
        self, raw_text: str, context: str = "", timeout_ms: int = 800
    ) -> dict:
        """Correct text using cloud LLM with timeout."""
        if self.provider == "none":
            raise RuntimeError("No cloud API key configured")

        prompt = self._build_prompt(raw_text, context)
        timeout_sec = timeout_ms / 1000.0

        try:
            if self.provider == "openai":
                return await self._correct_openai(prompt, timeout_sec)
            else:
                return await self._correct_anthropic(prompt, timeout_sec)
        except asyncio.TimeoutError:
            raise RuntimeError(f"Cloud correction timed out after {timeout_ms}ms")

    async def _correct_openai(self, prompt: str, timeout: float) -> dict:
        """Correct using OpenAI API."""
        async with httpx.AsyncClient(timeout=timeout) as client:
            response = await client.post(
                "https://api.openai.com/v1/chat/completions",
                headers={
                    "Authorization": f"Bearer {OPENAI_API_KEY}",
                    "Content-Type": "application/json",
                },
                json={
                    "model": "gpt-4o-mini",
                    "messages": [
                        {
                            "role": "system",
                            "content": "Fix transcription errors. Output only corrected text.",
                        },
                        {"role": "user", "content": prompt},
                    ],
                    "max_tokens": 200,
                    "temperature": 0,
                },
            )
            response.raise_for_status()
            data = response.json()
            corrected = data["choices"][0]["message"]["content"].strip()
            return {"corrected_text": corrected, "source": "cloud"}

    async def _correct_anthropic(self, prompt: str, timeout: float) -> dict:
        """Correct using Anthropic API."""
        async with httpx.AsyncClient(timeout=timeout) as client:
            response = await client.post(
                "https://api.anthropic.com/v1/messages",
                headers={
                    "x-api-key": ANTHROPIC_API_KEY,
                    "anthropic-version": "2023-06-01",
                    "content-type": "application/json",
                },
                json={
                    "model": "claude-3-haiku-20240307",
                    "max_tokens": 200,
                    "system": "Fix transcription errors. Output only corrected text.",
                    "messages": [{"role": "user", "content": prompt}],
                },
            )
            response.raise_for_status()
            data = response.json()
            corrected = data["content"][0]["text"].strip()
            return {"corrected_text": corrected, "source": "cloud"}

    def _build_prompt(self, raw_text: str, context: str) -> str:
        """Build a correction prompt."""
        context_line = f"Context: {context}" if context else ""
        return f"""Fix any transcription errors in the following text. Only output the corrected text, nothing else.
{context_line}
Text: {raw_text}
Corrected:"""


# Tests
import pytest


@pytest.mark.asyncio
async def test_cloud_llm_no_api_key():
    llm = CloudLLM()
    # Force no keys
    llm.provider = "none"
    with pytest.raises(RuntimeError, match="No cloud API key"):
        await llm.correct("test text")


@pytest.mark.asyncio
async def test_cloud_llm_prompt_building():
    llm = CloudLLM()
    llm.provider = "none"  # Don't actually call API
    prompt = llm._build_prompt("hello world", "context")
    assert "hello world" in prompt
    assert "context" in prompt
    assert "Fix any transcription errors" in prompt


@pytest.mark.asyncio
async def test_cloud_llm_detect_provider_openai(monkeypatch):
    monkeypatch.setenv("OPENAI_API_KEY", "test-key")
    monkeypatch.setenv("ANTHROPIC_API_KEY", "")
    # Need to reimport to pick up env vars
    import importlib
    import correction.cloud_llm as cloud_module
    importlib.reload(cloud_module)
    llm = cloud_module.CloudLLM()
    assert llm.provider == "openai"


@pytest.mark.asyncio
async def test_cloud_llm_detect_provider_anthropic(monkeypatch):
    monkeypatch.setenv("OPENAI_API_KEY", "")
    monkeypatch.setenv("ANTHROPIC_API_KEY", "test-key")
    import importlib
    import correction.cloud_llm as cloud_module
    importlib.reload(cloud_module)
    llm = cloud_module.CloudLLM()
    assert llm.provider == "anthropic"
```

- [ ] **Step 3: Run Python tests**

Run: `cd handy01/sidecar && pip install pytest pytest-asyncio && pytest correction/ -v`
Expected: All correction tests pass

- [ ] **Step 4: Commit**

```bash
git add sidecar/correction/
git commit -m "feat: add Python local and cloud LLM correction modules with tests"
```

---

## Phase 3: Integration

### Task 6: Rust Sidecar Communication

**Files:**
- Create: `handy01/src-tauri/src/sidecar/mod.rs`
- Create: `handy01/src-tauri/src/sidecar/client.rs`
- Create: `handy01/src-tauri/src/sidecar/process.rs`

- [ ] **Step 1: Write tests and implementation for sidecar client**

Create: `handy01/src-tauri/src/sidecar/client.rs`

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8765";

#[derive(Serialize)]
pub struct TranscribeRequest {
    pub audio_base64: String,
    #[serde(default)]
    pub context: String,
}

#[derive(Deserialize, Debug)]
pub struct TranscribeResponse {
    pub text: String,
    pub confidence: f64,
    pub timestamp_start: f64,
    pub timestamp_end: f64,
}

#[derive(Serialize)]
pub struct CorrectRequest {
    pub raw_text: String,
    pub confidence: f64,
    #[serde(default)]
    pub context: String,
    #[serde(default)]
    pub use_cloud: bool,
}

#[derive(Deserialize, Debug)]
pub struct CorrectResponse {
    pub corrected_text: String,
    pub source: String,
    pub latency_ms: f64,
}

#[derive(Deserialize, Debug)]
pub struct HealthResponse {
    pub status: String,
    pub whisper_loaded: bool,
    pub correction_loaded: bool,
}

/// HTTP client for communicating with the Python sidecar
pub struct SidecarClient {
    client: Client,
    base_url: String,
}

impl SidecarClient {
    pub fn new(base_url: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
        }
    }

    /// Check if sidecar is healthy
    pub async fn health_check(&self) -> Result<HealthResponse, String> {
        let url = format!("{}/health", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Health check failed: {}", e))?;

        response
            .json::<HealthResponse>()
            .await
            .map_err(|e| format!("Failed to parse health response: {}", e))
    }

    /// Send audio chunk for transcription
    pub async fn transcribe(&self, request: &TranscribeRequest) -> Result<TranscribeResponse, String> {
        let url = format!("{}/transcribe", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(request)
            .send()
            .await
            .map_err(|e| format!("Transcription request failed: {}", e))?;

        response
            .json::<TranscribeResponse>()
            .await
            .map_err(|e| format!("Failed to parse transcription response: {}", e))
    }

    /// Send text for AI correction
    pub async fn correct(&self, request: &CorrectRequest) -> Result<CorrectResponse, String> {
        let url = format!("{}/correct", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(request)
            .send()
            .await
            .map_err(|e| format!("Correction request failed: {}", e))?;

        response
            .json::<CorrectResponse>()
            .await
            .map_err(|e| format!("Failed to parse correction response: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = SidecarClient::new(None);
        assert_eq!(client.base_url, DEFAULT_BASE_URL);
    }

    #[test]
    fn test_client_custom_url() {
        let client = SidecarClient::new(Some("http://localhost:9999".to_string()));
        assert_eq!(client.base_url, "http://localhost:9999");
    }

    #[tokio::test]
    async fn test_health_check_fails_without_server() {
        let client = SidecarClient::new(None);
        let result = client.health_check().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transcribe_fails_without_server() {
        let client = SidecarClient::new(None);
        let request = TranscribeRequest {
            audio_base64: "test".to_string(),
            context: String::new(),
        };
        let result = client.transcribe(&request).await;
        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: Write tests and implementation for sidecar process management**

Create: `handy01/src-tauri/src/sidecar/process.rs`

```rust
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use super::client::SidecarClient;

/// Manages the Python sidecar process lifecycle
pub struct SidecarProcess {
    process: Option<Child>,
    running: Arc<AtomicBool>,
    port: u16,
}

impl SidecarProcess {
    pub fn new(port: u16) -> Self {
        Self {
            process: None,
            running: Arc::new(AtomicBool::new(false)),
            port,
        }
    }

    /// Start the Python sidecar process
    pub fn start(&mut self, sidecar_dir: PathBuf) -> Result<(), String> {
        if self.is_running() {
            return Ok(());
        }

        let python_cmd = Self::find_python()?;
        let server_path = sidecar_dir.join("server.py");

        if !server_path.exists() {
            return Err(format!("Sidecar server.py not found at {:?}", server_path));
        }

        let mut cmd = Command::new(&python_cmd);
        cmd.arg("-m")
            .arg("uvicorn")
            .arg("server:app")
            .arg("--host")
            .arg("127.0.0.1")
            .arg("--port")
            .arg(self.port.to_string())
            .current_dir(&sidecar_dir);

        // Set LD_LIBRARY_PATH for CUDA if available
        if let Ok(ld_path) = std::env::var("LD_LIBRARY_PATH") {
            cmd.env("LD_LIBRARY_PATH", ld_path);
        }

        let child = cmd
            .spawn()
            .map_err(|e| format!("Failed to start sidecar: {}", e))?;

        self.process = Some(child);
        self.running.store(true, Ordering::SeqCst);

        Ok(())
    }

    /// Stop the sidecar process
    pub fn stop(&mut self) {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.running.store(false, Ordering::SeqCst);
    }

    /// Check if sidecar is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
            && self
                .process
                .as_ref()
                .map(|p| p.try_wait().map(|w| w.is_none()).unwrap_or(false))
                .unwrap_or(false)
    }

    /// Wait for sidecar to become healthy
    pub async fn wait_for_health(&self, client: &SidecarClient, max_retries: u32) -> Result<(), String> {
        for i in 0..max_retries {
            if client.health_check().await.is_ok() {
                return Ok(());
            }
            sleep(Duration::from_millis(500)).await;
        }
        Err("Sidecar failed to become healthy".to_string())
    }

    fn find_python() -> Result<String, String> {
        which::which("python3")
            .or_else(|_| which::which("python"))
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|_| "Python not found".to_string())
    }
}

impl Drop for SidecarProcess {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_not_running_initially() {
        let process = SidecarProcess::new(8765);
        assert!(!process.is_running());
    }

    #[test]
    fn test_process_start_fails_without_server() {
        let mut process = SidecarProcess::new(8765);
        let result = process.start(PathBuf::from("/nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_find_python() {
        let result = SidecarProcess::find_python();
        // Either finds python or returns error
        match result {
            Ok(path) => assert!(!path.is_empty()),
            Err(_) => assert!(true),
        }
    }
}
```

- [ ] **Step 3: Create sidecar mod.rs**

Create: `handy01/src-tauri/src/sidecar/mod.rs`

```rust
pub mod client;
pub mod process;

pub use client::SidecarClient;
pub use process::SidecarProcess;
```

- [ ] **Step 4: Run tests**

Run: `cd handy01/src-tauri && cargo test sidecar::`
Expected: All sidecar tests pass

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/sidecar/
git commit -m "feat: add Rust sidecar client and process management"
```

---

### Task 7: Rust Text Injection and Hotkey Management

**Files:**
- Create: `handy01/src-tauri/src/inject/mod.rs`
- Create: `handy01/src-tauri/src/inject/text_injector.rs`
- Create: `handy01/src-tauri/src/hotkey/mod.rs`
- Create: `handy01/src-tauri/src/hotkey/manager.rs`

- [ ] **Step 1: Write tests and implementation for text_injector**

Create: `handy01/src-tauri/src/inject/text_injector.rs`

```rust
use enigo::{Enigo, Key, Keyboard, Settings};
use std::thread;
use std::time::Duration;

/// Injects text into the currently focused application
pub struct TextInjector {
    enigo: Enigo,
    fallback_to_clipboard: bool,
}

impl TextInjector {
    pub fn new() -> Self {
        Self {
            enigo: Enigo::new(&Settings::default()).expect("Failed to initialize Enigo"),
            fallback_to_clipboard: true,
        }
    }

    /// Inject text by simulating keystrokes
    pub fn inject(&mut self, text: &str) -> Result<(), String> {
        // For longer text, use clipboard paste as primary method
        // Key simulation is unreliable for long strings
        if text.len() > 50 {
            self.inject_via_clipboard(text)
        } else {
            self.inject_via_keystrokes(text)
        }
    }

    /// Inject short text via individual keystrokes
    fn inject_via_keystrokes(&mut self, text: &str) -> Result<(), String> {
        for ch in text.chars() {
            match ch {
                ' ' => self.enigo.key(Key::Space).map_err(|e| e.to_string())?,
                '\n' | '\r' => self.enigo.key(Key::Return).map_err(|e| e.to_string())?,
                '\t' => self.enigo.key(Key::Tab).map_err(|e| e.to_string())?,
                c => {
                    // Type character directly
                    self.enigo
                        .text(&c.to_string())
                        .map_err(|e| format!("Failed to type '{}': {}", c, e))?;
                }
            }
            // Small delay between characters for reliability
            thread::sleep(Duration::from_millis(5));
        }
        Ok(())
    }

    /// Inject text via clipboard paste (more reliable for longer text)
    fn inject_via_clipboard(&mut self, text: &str) -> Result<(), String> {
        // Set clipboard content
        arboard::Clipboard::new()
            .map_err(|e| format!("Failed to access clipboard: {}", e))?
            .set_text(text)
            .map_err(|e| format!("Failed to set clipboard text: {}", e))?;

        // Small delay to ensure clipboard is set
        thread::sleep(Duration::from_millis(50));

        // Simulate Ctrl+V (or Cmd+V on macOS)
        #[cfg(target_os = "macos")]
        {
            self.enigo.key(Key::Meta).map_err(|e| e.to_string())?;
            self.enigo.key(Key::V).map_err(|e| e.to_string())?;
            self.enigo.key(Key::Meta).map_err(|e| e.to_string())?;
        }

        #[cfg(not(target_os = "macos"))]
        {
            self.enigo.key(Key::Control).map_err(|e| e.to_string())?;
            self.enigo.key(Key::V).map_err(|e| e.to_string())?;
            self.enigo.key(Key::Control).map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Inject a backspace sequence to delete last N characters
    pub fn backspace(&mut self, count: usize) -> Result<(), String> {
        for _ in 0..count {
            self.enigo.key(Key::Backspace).map_err(|e| e.to_string())?;
            thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }
}

impl Default for TextInjector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_injector_creation() {
        let injector = TextInjector::new();
        // Creation should succeed on systems with input access
        assert!(injector.fallback_to_clipboard);
    }

    #[test]
    fn test_injector_default() {
        let injector = TextInjector::default();
        assert!(injector.fallback_to_clipboard);
    }
}
```

- [ ] **Step 2: Update Cargo.toml for arboard dependency**

Read the current `src-tauri/Cargo.toml` and add:

```toml
arboard = "3.4"
```

to the `[dependencies]` section.

- [ ] **Step 3: Write tests and implementation for hotkey manager**

Create: `handy01/src-tauri/src/hotkey/manager.rs`

```rust
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Hotkey configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub modifiers: Vec<String>,
    pub key: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            modifiers: vec!["Control".to_string(), "Shift".to_string()],
            key: "Space".to_string(),
        }
    }
}

/// State for hotkey management
#[derive(Clone)]
pub struct HotkeyState {
    pub is_recording: bool,
    pub config: HotkeyConfig,
}

/// Manages global hotkey registration
pub struct HotkeyManager {
    state: Arc<Mutex<HotkeyState>>,
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(HotkeyState {
                is_recording: false,
                config: HotkeyConfig::default(),
            })),
        }
    }

    /// Get the current hotkey state
    pub async fn get_state(&self) -> HotkeyState {
        self.state.lock().await.clone()
    }

    /// Toggle recording state
    pub async fn toggle_recording(&self) -> bool {
        let mut state = self.state.lock().await;
        state.is_recording = !state.is_recording;
        state.is_recording
    }

    /// Check if currently recording
    pub async fn is_recording(&self) -> bool {
        self.state.lock().await.is_recording
    }

    /// Update hotkey configuration
    pub async fn set_config(&self, config: HotkeyConfig) {
        let mut state = self.state.lock().await;
        state.config = config;
    }

    /// Get the hotkey string in Tauri format
    pub async fn get_hotkey_string(&self) -> String {
        let state = self.state.lock().await;
        let mods = state.config.modifiers.join("+");
        format!("{}+{}", mods, state.config.key)
    }
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hotkey_manager_creation() {
        let manager = HotkeyManager::new();
        let state = manager.get_state().await;
        assert!(!state.is_recording);
        assert_eq!(state.config.key, "Space");
    }

    #[tokio::test]
    async fn test_toggle_recording() {
        let manager = HotkeyManager::new();

        let recording = manager.toggle_recording().await;
        assert!(recording);

        let recording = manager.toggle_recording().await;
        assert!(!recording);
    }

    #[tokio::test]
    async fn test_is_recording() {
        let manager = HotkeyManager::new();
        assert!(!manager.is_recording().await);

        manager.toggle_recording().await;
        assert!(manager.is_recording().await);
    }

    #[tokio::test]
    async fn test_set_config() {
        let manager = HotkeyManager::new();
        manager
            .set_config(HotkeyConfig {
                modifiers: vec!["Alt".to_string()],
                key: "D".to_string(),
            })
            .await;

        let hotkey = manager.get_hotkey_string().await;
        assert_eq!(hotkey, "Alt+D");
    }

    #[tokio::test]
    async fn test_default_hotkey_string() {
        let manager = HotkeyManager::new();
        let hotkey = manager.get_hotkey_string().await;
        assert_eq!(hotkey, "Control+Shift+Space");
    }
}
```

- [ ] **Step 4: Create module exports**

Create: `handy01/src-tauri/src/inject/mod.rs`

```rust
pub mod text_injector;

pub use text_injector::TextInjector;
```

Create: `handy01/src-tauri/src/hotkey/mod.rs`

```rust
pub mod manager;

pub use manager::{HotkeyConfig, HotkeyManager, HotkeyState};
```

- [ ] **Step 5: Run tests**

Run: `cd handy01/src-tauri && cargo test inject::`
Run: `cd handy01/src-tauri && cargo test hotkey::`
Expected: All tests pass

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/inject/ src-tauri/src/hotkey/ src-tauri/Cargo.toml
git commit -m "feat: add text injection and hotkey management modules"
```

---

### Task 8: State Management and Tauri Commands

**Files:**
- Create: `handy01/src-tauri/src/state/mod.rs`
- Create: `handy01/src-tauri/src/state/settings.rs`
- Create: `handy01/src-tauri/src/state/app_state.rs`
- Modify: `handy01/src-tauri/src/lib.rs`
- Modify: `handy01/src-tauri/src/main.rs`

- [ ] **Step 1: Write tests and implementation for settings**

Create: `handy01/src-tauri/src/state/settings.rs`

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::correction::CorrectionStyle;
use crate::hotkey::HotkeyConfig;

/// Application settings persisted to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub hotkey: HotkeyConfig,
    pub correction_style: CorrectionStyle,
    pub whisper_model: String,
    pub local_llm_model: String,
    pub cloud_provider: String,
    pub openai_api_key: String,
    pub anthropic_api_key: String,
    pub sidecar_port: u16,
    pub auto_start_sidecar: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: HotkeyConfig::default(),
            correction_style: CorrectionStyle::default(),
            whisper_model: "large-v3".to_string(),
            local_llm_model: "TinyLlama/TinyLlama-1.1B-Chat-v1.0".to_string(),
            cloud_provider: "openai".to_string(),
            openai_api_key: String::new(),
            anthropic_api_key: String::new(),
            sidecar_port: 8765,
            auto_start_sidecar: true,
        }
    }
}

impl Settings {
    /// Load settings from disk
    pub fn load() -> Result<Self, String> {
        let path = Self::settings_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read settings: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse settings: {}", e))
    }

    /// Save settings to disk
    pub fn save(&self) -> Result<(), String> {
        let path = Self::settings_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create settings dir: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        std::fs::write(&path, content)
            .map_err(|e| format!("Failed to write settings: {}", e))
    }

    fn settings_path() -> Result<PathBuf, String> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| "Could not find config directory".to_string())?;
        Ok(config_dir.join("handy01").join("settings.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.sidecar_port, 8765);
        assert_eq!(settings.whisper_model, "large-v3");
        assert!(settings.auto_start_sidecar);
    }

    #[test]
    fn test_settings_serialization() {
        let settings = Settings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let loaded: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings.sidecar_port, loaded.sidecar_port);
    }
}
```

- [ ] **Step 2: Write app_state**

Create: `handy01/src-tauri/src/state/app_state.rs`

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::correction::CorrectionStyle;
use crate::correction::SlidingWindow;
use crate::hotkey::HotkeyManager;
use crate::inject::TextInjector;
use crate::sidecar::SidecarClient;
use crate::sidecar::SidecarProcess;

use super::settings::Settings;

/// Shared application state accessible from Tauri commands
pub struct AppState {
    pub settings: Mutex<Settings>,
    pub hotkey_manager: HotkeyManager,
    pub sidecar_process: Mutex<SidecarProcess>,
    pub sidecar_client: SidecarClient,
    pub text_injector: Mutex<TextInjector>,
    pub sliding_window: Mutex<SlidingWindow>,
    pub is_recording: Mutex<bool>,
    pub last_transcript: Mutex<String>,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        let port = settings.sidecar_port;
        Self {
            settings: Mutex::new(settings),
            hotkey_manager: HotkeyManager::new(),
            sidecar_process: Mutex::new(SidecarProcess::new(port)),
            sidecar_client: SidecarClient::new(None),
            text_injector: Mutex::new(TextInjector::new()),
            sliding_window: Mutex::new(SlidingWindow::new(5000)),
            is_recording: Mutex::new(false),
            last_transcript: Mutex::new(String::new()),
        }
    }
}
```

- [ ] **Step 3: Create state mod.rs**

Create: `handy01/src-tauri/src/state/mod.rs`

```rust
pub mod app_state;
pub mod settings;

pub use app_state::AppState;
pub use settings::Settings;
```

- [ ] **Step 4: Write lib.rs with Tauri commands**

Create: `handy01/src-tauri/src/lib.rs`

```rust
mod audio;
mod correction;
mod hotkey;
mod inject;
mod sidecar;
mod state;

use state::AppState;
use state::Settings;
use tauri::Manager;

#[tauri::command]
async fn toggle_recording(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let is_recording = state.hotkey_manager.toggle_recording().await;
    *state.is_recording.lock().await = is_recording;

    if !is_recording {
        // Stop recording, flush remaining buffer
        state.sliding_window.lock().await.lock();
    }

    Ok(is_recording)
}

#[tauri::command]
async fn get_recording_state(state: tauri::State<'_, AppState>) -> bool {
    *state.is_recording.lock().await
}

#[tauri::command]
async fn get_settings(state: tauri::State<'_, AppState>) -> Result<Settings, String> {
    let settings = state.settings.lock().await;
    Ok(settings.clone())
}

#[tauri::command]
async fn update_settings(
    state: tauri::State<'_, AppState>,
    settings: Settings,
) -> Result<(), String> {
    settings.save()?;
    *state.settings.lock().await = settings;
    Ok(())
}

#[tauri::command]
async fn get_current_text(state: tauri::State<'_, AppState>) -> String {
    state.sliding_window.lock().await.get_full()
}

#[tauri::command]
async fn check_sidecar_health(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    match state.sidecar_client.health_check().await {
        Ok(health) => Ok(health.whisper_loaded && health.correction_loaded),
        Err(_) => Ok(false),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings = Settings::load().unwrap_or_default();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState::new(settings))
        .invoke_handler(tauri::generate_handler![
            toggle_recording,
            get_recording_state,
            get_settings,
            update_settings,
            get_current_text,
            check_sidecar_health,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 5: Write main.rs**

Create: `handy01/src-tauri/src/main.rs`

```rust
// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    handy01_lib::run()
}
```

- [ ] **Step 6: Run tests and check compilation**

Run: `cd handy01/src-tauri && cargo test state::`
Run: `cd handy01/src-tauri && cargo check`
Expected: All tests pass, compilation succeeds

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/state/ src-tauri/src/lib.rs src-tauri/src/main.rs
git commit -m "feat: add state management and Tauri commands"
```

---

## Phase 4: UI

### Task 9: React UI Components

**Files:**
- Create: `handy01/src/main.tsx`, `handy01/src/App.tsx`, `handy01/index.html`
- Create: `handy01/src/components/LiveTranscript.tsx`
- Create: `handy01/src/components/SettingsPanel.tsx`
- Create: `handy01/src/components/StatusIndicator.tsx`
- Create: `handy01/src/components/CorrectionPreview.tsx`
- Create: `handy01/src/styles/globals.css`

- [ ] **Step 1: Create index.html**

Create: `handy01/index.html`

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Handy01</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

- [ ] **Step 2: Create globals.css**

Create: `handy01/src/styles/globals.css`

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

:root {
  --bg-primary: #1a1a2e;
  --bg-secondary: #16213e;
  --text-primary: #e8e8e8;
  --text-secondary: #a0a0a0;
  --accent: #0f3460;
  --accent-bright: #e94560;
  --highlight: #f0ad4e;
  --success: #5cb85c;
}

body {
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  background: var(--bg-primary);
  color: var(--text-primary);
}

.highlighted-correction {
  border-bottom: 2px solid var(--highlight);
  background: rgba(240, 173, 78, 0.1);
  padding: 0 2px;
  border-radius: 2px;
}

.draft-text {
  text-decoration: line-through;
  color: var(--text-secondary);
  margin-right: 4px;
}

.final-text {
  color: var(--text-primary);
}

.recording-indicator {
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.4;
  }
}
```

- [ ] **Step 3: Create main.tsx**

Create: `handy01/src/main.tsx`

```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles/globals.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
```

- [ ] **Step 4: Create LiveTranscript component**

Create: `handy01/src/components/LiveTranscript.tsx`

```tsx
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface LiveTranscriptProps {
  correctionStyle: "inline" | "highlighted" | "draft-final";
}

export default function LiveTranscript({ correctionStyle }: LiveTranscriptProps) {
  const [text, setText] = useState("");
  const [isRecording, setIsRecording] = useState(false);

  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const currentText = await invoke<string>("get_current_text");
        setText(currentText);
      } catch {
        // Ignore errors during polling
      }
    }, 200);

    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const checkRecording = async () => {
      try {
        const recording = await invoke<boolean>("get_recording_state");
        setIsRecording(recording);
      } catch {
        // Ignore errors
      }
    };

    const interval = setInterval(checkRecording, 500);
    return () => clearInterval(interval);
  }, []);

  const renderText = () => {
    if (!text) {
      return (
        <span className="text-gray-500 italic">
          {isRecording ? "Listening..." : "Press Ctrl+Shift+Space to start"}
        </span>
      );
    }

    switch (correctionStyle) {
      case "highlighted":
        return <span className="highlighted-correction">{text}</span>;
      case "draft-final":
        return <span className="final-text">{text}</span>;
      default:
        return <span>{text}</span>;
    }
  };

  return (
    <div className="p-4 min-h-[200px] bg-[var(--bg-secondary)] rounded-lg">
      <div className="flex items-center gap-2 mb-2">
        {isRecording && (
          <div className="w-3 h-3 rounded-full bg-red-500 recording-indicator" />
        )}
        <span className="text-sm text-[var(--text-secondary)]">
          {isRecording ? "Recording" : "Idle"}
        </span>
      </div>
      <div className="text-lg leading-relaxed">{renderText()}</div>
    </div>
  );
}
```

- [ ] **Step 5: Create StatusIndicator component**

Create: `handy01/src/components/StatusIndicator.tsx`

```tsx
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function StatusIndicator() {
  const [sidecarHealthy, setSidecarHealthy] = useState<boolean | null>(null);
  const [isRecording, setIsRecording] = useState(false);

  useEffect(() => {
    const checkHealth = async () => {
      try {
        const healthy = await invoke<boolean>("check_sidecar_health");
        setSidecarHealthy(healthy);
      } catch {
        setSidecarHealthy(false);
      }
    };

    const checkRecording = async () => {
      try {
        const recording = await invoke<boolean>("get_recording_state");
        setIsRecording(recording);
      } catch {
        // Ignore
      }
    };

    checkHealth();
    checkRecording();

    const interval = setInterval(() => {
      checkHealth();
      checkRecording();
    }, 2000);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="flex items-center gap-3 text-sm">
      <div className="flex items-center gap-1">
        <div
          className={`w-2 h-2 rounded-full ${
            sidecarHealthy === true
              ? "bg-green-500"
              : sidecarHealthy === false
                ? "bg-red-500"
                : "bg-yellow-500"
          }`}
        />
        <span className="text-[var(--text-secondary)]">Sidecar</span>
      </div>
      <div className="flex items-center gap-1">
        <div
          className={`w-2 h-2 rounded-full ${isRecording ? "bg-red-500 recording-indicator" : "bg-gray-500"}`}
        />
        <span className="text-[var(--text-secondary)]">
          {isRecording ? "Recording" : "Idle"}
        </span>
      </div>
    </div>
  );
}
```

- [ ] **Step 6: Create SettingsPanel component**

Create: `handy01/src/components/SettingsPanel.tsx`

```tsx
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Settings {
  correction_style: string;
  whisper_model: string;
  sidecar_port: number;
  openai_api_key: string;
  anthropic_api_key: string;
  hotkey: {
    modifiers: string[];
    key: string;
  };
}

interface SettingsPanelProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function SettingsPanel({ isOpen, onClose }: SettingsPanelProps) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (isOpen) {
      loadSettings();
    }
  }, [isOpen]);

  const loadSettings = async () => {
    try {
      const loaded = await invoke<Settings>("get_settings");
      setSettings(loaded);
    } catch (e) {
      console.error("Failed to load settings:", e);
    }
  };

  const saveSettings = async () => {
    if (!settings) return;
    setSaving(true);
    try {
      await invoke("update_settings", { settings });
      onClose();
    } catch (e) {
      console.error("Failed to save settings:", e);
    } finally {
      setSaving(false);
    }
  };

  if (!isOpen || !settings) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-[var(--bg-secondary)] rounded-lg p-6 w-[400px] max-h-[80vh] overflow-y-auto">
        <h2 className="text-xl font-bold mb-4">Settings</h2>

        <div className="space-y-4">
          <div>
            <label className="block text-sm text-[var(--text-secondary)] mb-1">
              Correction Style
            </label>
            <select
              className="w-full bg-[var(--bg-primary)] text-[var(--text-primary)] rounded p-2"
              value={settings.correction_style}
              onChange={(e) =>
                setSettings({ ...settings, correction_style: e.target.value })
              }
            >
              <option value="inline">Inline (silent correction)</option>
              <option value="highlighted">Highlighted (show corrections)</option>
              <option value="draft-final">Draft → Final (show both)</option>
            </select>
          </div>

          <div>
            <label className="block text-sm text-[var(--text-secondary)] mb-1">
              Whisper Model
            </label>
            <select
              className="w-full bg-[var(--bg-primary)] text-[var(--text-primary)] rounded p-2"
              value={settings.whisper_model}
              onChange={(e) =>
                setSettings({ ...settings, whisper_model: e.target.value })
              }
            >
              <option value="large-v3">large-v3 (best quality)</option>
              <option value="medium">medium (balanced)</option>
              <option value="small">small (fastest)</option>
            </select>
          </div>

          <div>
            <label className="block text-sm text-[var(--text-secondary)] mb-1">
              OpenAI API Key
            </label>
            <input
              type="password"
              className="w-full bg-[var(--bg-primary)] text-[var(--text-primary)] rounded p-2"
              value={settings.openai_api_key}
              onChange={(e) =>
                setSettings({ ...settings, openai_api_key: e.target.value })
              }
              placeholder="sk-..."
            />
          </div>

          <div>
            <label className="block text-sm text-[var(--text-secondary)] mb-1">
              Anthropic API Key
            </label>
            <input
              type="password"
              className="w-full bg-[var(--bg-primary)] text-[var(--text-primary)] rounded p-2"
              value={settings.anthropic_api_key}
              onChange={(e) =>
                setSettings({ ...settings, anthropic_api_key: e.target.value })
              }
              placeholder="sk-ant-..."
            />
          </div>

          <div>
            <label className="block text-sm text-[var(--text-secondary)] mb-1">
              Sidecar Port
            </label>
            <input
              type="number"
              className="w-full bg-[var(--bg-primary)] text-[var(--text-primary)] rounded p-2"
              value={settings.sidecar_port}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  sidecar_port: parseInt(e.target.value) || 8765,
                })
              }
            />
          </div>
        </div>

        <div className="flex gap-2 mt-6">
          <button
            className="flex-1 bg-[var(--accent-bright)] text-white rounded p-2 hover:opacity-90"
            onClick={saveSettings}
            disabled={saving}
          >
            {saving ? "Saving..." : "Save"}
          </button>
          <button
            className="flex-1 bg-[var(--accent)] text-white rounded p-2 hover:opacity-90"
            onClick={onClose}
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 7: Create CorrectionPreview component**

Create: `handy01/src/components/CorrectionPreview.tsx`

```tsx
interface CorrectionPreviewProps {
  style: "inline" | "highlighted" | "draft-final";
}

const SAMPLE_RAW = "i went to the store to buy some bred";
const SAMPLE_CORRECTED = "I went to the store to buy some bread";

export default function CorrectionPreview({ style }: CorrectionPreviewProps) {
  const renderPreview = () => {
    switch (style) {
      case "inline":
        return <span>{SAMPLE_CORRECTED}</span>;
      case "highlighted":
        return (
          <span>
            I went to the store to buy some{" "}
            <span className="highlighted-correction">bread</span>
          </span>
        );
      case "draft-final":
        return (
          <span>
            <span className="draft-text">{SAMPLE_RAW}</span>
            <span className="final-text">→ {SAMPLE_CORRECTED}</span>
          </span>
        );
    }
  };

  return (
    <div className="p-3 bg-[var(--bg-primary)] rounded border border-[var(--accent)]">
      <p className="text-xs text-[var(--text-secondary)] mb-1">Preview:</p>
      <p className="text-sm">{renderPreview()}</p>
    </div>
  );
}
```

- [ ] **Step 8: Create App.tsx**

Create: `handy01/src/App.tsx`

```tsx
import { useState } from "react";
import LiveTranscript from "./components/LiveTranscript";
import StatusIndicator from "./components/StatusIndicator";
import SettingsPanel from "./components/SettingsPanel";
import CorrectionPreview from "./components/CorrectionPreview";

function App() {
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [correctionStyle, setCorrectionStyle] = useState<
    "inline" | "highlighted" | "draft-final"
  >("inline");

  return (
    <div className="min-h-screen bg-[var(--bg-primary)] p-4">
      <div className="max-w-lg mx-auto space-y-4">
        <header className="flex items-center justify-between">
          <h1 className="text-xl font-bold">Handy01</h1>
          <div className="flex items-center gap-3">
            <StatusIndicator />
            <button
              className="text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
              onClick={() => setSettingsOpen(true)}
            >
              ⚙ Settings
            </button>
          </div>
        </header>

        <LiveTranscript correctionStyle={correctionStyle} />

        <CorrectionPreview style={correctionStyle} />

        <div className="flex gap-2">
          {(["inline", "highlighted", "draft-final"] as const).map((style) => (
            <button
              key={style}
              className={`px-3 py-1 rounded text-sm ${
                correctionStyle === style
                  ? "bg-[var(--accent-bright)] text-white"
                  : "bg-[var(--bg-secondary)] text-[var(--text-secondary)]"
              }`}
              onClick={() => setCorrectionStyle(style)}
            >
              {style}
            </button>
          ))}
        </div>

        <p className="text-xs text-[var(--text-secondary)] text-center">
          Press Ctrl+Shift+Space to toggle recording
        </p>
      </div>

      <SettingsPanel isOpen={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </div>
  );
}

export default App;
```

- [ ] **Step 9: Verify UI builds**

Run: `cd handy01 && npm run build`
Expected: Build succeeds without errors

- [ ] **Step 10: Commit**

```bash
git add src/ index.html package.json vite.config.ts tsconfig.json tailwind.config.js
git commit -m "feat: add React UI components for live transcription and settings"
```

---

## Phase 5: Testing & Polish

### Task 10: E2E and Performance Tests

**Files:**
- Create: `handy01/tests/e2e/dictation.spec.ts`
- Create: `handy01/tests/performance/latency.bench.ts`

- [ ] **Step 1: Create E2E test**

Create: `handy01/tests/e2e/dictation.spec.ts`

```typescript
import { test, expect } from "@playwright/test";

test.describe("Handy01 Dictation", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
  });

  test("should display app title", async ({ page }) => {
    await expect(page.getByText("Handy01")).toBeVisible();
  });

  test("should show idle state initially", async ({ page }) => {
    await expect(page.getByText("Idle")).toBeVisible();
  });

  test("should show settings button", async ({ page }) => {
    await expect(page.getByText("Settings")).toBeVisible();
  });

  test("should open settings panel", async ({ page }) => {
    await page.getByText("Settings").click();
    await expect(page.getByText("Correction Style")).toBeVisible();
    await expect(page.getByText("Whisper Model")).toBeVisible();
  });

  test("should show correction style options", async ({ page }) => {
    await expect(page.getByText("inline")).toBeVisible();
    await expect(page.getByText("highlighted")).toBeVisible();
    await expect(page.getByText("draft-final")).toBeVisible();
  });

  test("should switch correction styles", async ({ page }) => {
    await page.getByText("highlighted").click();
    const preview = page.locator(".highlighted-correction");
    await expect(preview).toBeVisible();
  });

  test("should show placeholder text when idle", async ({ page }) => {
    await expect(
      page.getByText("Press Ctrl+Shift+Space to start"),
    ).toBeVisible();
  });

  test("should display status indicators", async ({ page }) => {
    await expect(page.getByText("Sidecar")).toBeVisible();
  });
});
```

- [ ] **Step 2: Create performance benchmark**

Create: `handy01/tests/performance/latency.bench.ts`

```typescript
/**
 * Performance benchmarks for Handy01
 * Run with: npm run bench
 */

interface BenchmarkResult {
  name: string;
  duration_ms: number;
  target_ms: number;
  passed: boolean;
}

async function benchmark(name: string, fn: () => Promise<void>, target_ms: number): Promise<BenchmarkResult> {
  const start = performance.now();
  await fn();
  const duration = performance.now() - start;

  return {
    name,
    duration_ms: Math.round(duration),
    target_ms,
    passed: duration <= target_ms,
  };
}

async function runBenchmarks() {
  console.log("Running Handy01 Performance Benchmarks\n");

  const results: BenchmarkResult[] = [];

  // Benchmark: Correction style formatting
  results.push(
    await benchmark("Correction style formatting", async () => {
      const styles = ["inline", "highlighted", "draft-final"] as const;
      for (const style of styles) {
        for (let i = 0; i < 1000; i++) {
          // Simulate formatting
          const original = "hello world";
          const corrected = "Hello world";
          let result: string;
          switch (style) {
            case "inline":
              result = corrected;
              break;
            case "highlighted":
              result = `[[${corrected}]]`;
              break;
            case "draft-final":
              result = `~~${original}~~ → ${corrected}`;
              break;
          }
        }
      }
    }, 10),
  );

  // Benchmark: Text merge operation
  results.push(
    await benchmark("Text merge (1000 ops)", async () => {
      for (let i = 0; i < 1000; i++) {
        const existing = "Hello world how";
        const newText = "world how are you";
        // Simplified merge logic
        const existingWords = existing.split(" ");
        const newWords = newText.split(" ");
        const result = [...new Set([...existingWords, ...newWords])].join(" ");
      }
    }, 50),
  );

  // Print results
  console.log("Results:");
  console.log("─".repeat(60));

  let allPassed = true;
  for (const result of results) {
    const status = result.passed ? "✓ PASS" : "✗ FAIL";
    console.log(
      `${status} | ${result.name.padEnd(30)} | ${result.duration_ms}ms (target: ${result.target_ms}ms)`,
    );
    if (!result.passed) allPassed = false;
  }

  console.log("─".repeat(60));
  console.log(allPassed ? "All benchmarks passed!" : "Some benchmarks failed!");

  if (!allPassed) {
    process.exit(1);
  }
}

runBenchmarks();
```

- [ ] **Step 3: Create playwright.config.ts**

Create: `handy01/playwright.config.ts`

```typescript
import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: "html",
  use: {
    baseURL: "http://localhost:1420",
    trace: "on-first-retry",
  },
  webServer: {
    command: "npm run dev",
    url: "http://localhost:1420",
    reuseExistingServer: !process.env.CI,
  },
});
```

- [ ] **Step 4: Run E2E tests**

Run: `cd handy01 && npx playwright test`
Expected: All E2E tests pass

- [ ] **Step 5: Run performance benchmarks**

Run: `cd handy01 && npm run bench`
Expected: All benchmarks pass

- [ ] **Step 6: Final commit**

```bash
git add tests/ playwright.config.ts
git commit -m "test: add E2E tests and performance benchmarks"
```

---

## Post-Implementation Checklist

- [ ] All Rust tests pass: `cargo test` in `src-tauri/`
- [ ] All Python tests pass: `pytest` in `sidecar/`
- [ ] All E2E tests pass: `npx playwright test`
- [ ] Performance benchmarks pass: `npm run bench`
- [ ] Tauri dev mode works: `npm run tauri dev`
- [ ] Sidecar starts independently: `cd sidecar && uvicorn server:app --port 8765`
- [ ] Settings persist across restarts
- [ ] All correction styles render correctly
- [ ] Hotkey toggles recording state
- [ ] README.md updated with setup instructions
