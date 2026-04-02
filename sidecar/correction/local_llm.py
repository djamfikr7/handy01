"""Local LLM correction using small models (Phi-3-mini, TinyLlama)."""

import logging
import os

logger = logging.getLogger(__name__)

DEFAULT_LOCAL_MODEL = os.getenv("LOCAL_LLM_MODEL", "TinyLlama/TinyLlama-1.1B-Chat-v1.0")


class LocalLLM:
    def __init__(self, model_name: str = DEFAULT_LOCAL_MODEL):
        self.model_name = model_name
        self.pipeline = None
        self._load_model()

    def _load_model(self):
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
        context_line = f"Context: {context}" if context else ""
        return f"""Fix any transcription errors in the following text. Only output the corrected text, nothing else.
{context_line}
Text: {raw_text}
Corrected:"""

    def _extract_correction(self, generated: str, prompt: str) -> str:
        if generated.startswith(prompt):
            return generated[len(prompt) :].strip()
        return generated.strip()
