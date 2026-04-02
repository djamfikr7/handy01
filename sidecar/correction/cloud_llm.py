"""Cloud LLM correction using OpenAI or Anthropic APIs."""

import asyncio
import logging
import os

import httpx

logger = logging.getLogger(__name__)

OPENAI_API_KEY = os.getenv("OPENAI_API_KEY", "")
ANTHROPIC_API_KEY = os.getenv("ANTHROPIC_API_KEY", "")
DEFAULT_CLOUD_MODEL = os.getenv("CLOUD_MODEL", "gpt-4o-mini")


class CloudLLM:
    def __init__(self):
        self.provider = self._detect_provider()

    def _detect_provider(self) -> str:
        if ANTHROPIC_API_KEY:
            return "anthropic"
        if OPENAI_API_KEY:
            return "openai"
        return "none"

    async def correct(
        self, raw_text: str, context: str = "", timeout_ms: int = 800
    ) -> dict:
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
        context_line = f"Context: {context}" if context else ""
        return f"""Fix any transcription errors in the following text. Only output the corrected text, nothing else.
{context_line}
Text: {raw_text}
Corrected:"""
