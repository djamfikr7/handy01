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
        model_path = self.cache_dir / "whisper" / model_size
        if model_path.exists():
            logger.info(f"Whisper model {model_size} found in cache")
            return str(model_path)

        logger.info(f"Downloading Whisper model {model_size}...")
        repo_id = f"guillaumekln/faster-whisper-{model_size}"
        local_dir = snapshot_download(repo_id, local_dir=str(model_path))
        logger.info(f"Whisper model downloaded to {local_dir}")
        return local_dir

    def ensure_local_llm_model(
        self, model_name: str = "TinyLlama/TinyLlama-1.1B-Chat-v1.0"
    ) -> str:
        model_path = self.cache_dir / "llm" / model_name.replace("/", "_")
        if model_path.exists():
            logger.info(f"LLM model {model_name} found in cache")
            return str(model_path)

        logger.info(f"Downloading LLM model {model_name}...")
        local_dir = snapshot_download(model_name, local_dir=str(model_path))
        logger.info(f"LLM model downloaded to {local_dir}")
        return local_dir
