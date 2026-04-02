"""Whisper transcription module using faster-whisper."""

import base64
import logging
import os
import tempfile

from faster_whisper import WhisperModel

logger = logging.getLogger(__name__)

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
            confidence_normalized = min(1.0, max(0.0, 1.0 + avg_confidence))

            return {
                "text": " ".join(texts),
                "confidence": round(confidence_normalized, 3),
                "timestamp_start": start_ts,
                "timestamp_end": end_ts,
            }
        finally:
            os.unlink(temp_path)
