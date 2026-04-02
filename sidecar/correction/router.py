"""Hybrid correction router - decides between local, cloud, or pass-through correction."""

import logging
import time

from .local_llm import LocalLLM
from .cloud_llm import CloudLLM

logger = logging.getLogger(__name__)

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
        if not raw_text.strip():
            return {
                "corrected_text": raw_text,
                "source": "pass-through",
                "latency_ms": 0.0,
            }

        if confidence >= HIGH_CONFIDENCE_THRESHOLD and not force_cloud:
            return {
                "corrected_text": raw_text,
                "source": "pass-through",
                "latency_ms": 0.0,
            }

        start_time = time.time()

        if confidence >= MEDIUM_CONFIDENCE_THRESHOLD:
            try:
                result = await self.local_llm.correct(raw_text, context)
                latency_ms = (time.time() - start_time) * 1000
                return {**result, "latency_ms": round(latency_ms, 1)}
            except Exception as e:
                logger.warning(f"Local LLM failed, falling back: {e}")

        if force_cloud or confidence < MEDIUM_CONFIDENCE_THRESHOLD:
            try:
                result = await self.cloud_llm.correct(
                    raw_text, context, timeout_ms=CLOUD_TIMEOUT_MS
                )
                latency_ms = (time.time() - start_time) * 1000
                return {**result, "latency_ms": round(latency_ms, 1)}
            except Exception as e:
                logger.warning(f"Cloud LLM failed, falling back: {e}")

        result = self._rule_based_correct(raw_text)
        latency_ms = (time.time() - start_time) * 1000
        return {
            "corrected_text": result,
            "source": "rule-based",
            "latency_ms": round(latency_ms, 1),
        }

    def _rule_based_correct(self, text: str) -> str:
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

        if result:
            result = result[0].upper() + result[1:]

        return result
