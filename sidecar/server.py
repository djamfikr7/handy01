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
    source: str
    latency_ms: float


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
