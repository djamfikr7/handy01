import pytest
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


@pytest.mark.asyncio
async def test_local_llm_prompt_building():
    from correction.local_llm import LocalLLM

    llm = LocalLLM.__new__(LocalLLM)
    llm.pipeline = None

    prompt = llm._build_prompt("hello world", "previous context")
    assert "hello world" in prompt
    assert "previous context" in prompt
    assert "Fix any transcription errors" in prompt


@pytest.mark.asyncio
async def test_local_llm_prompt_no_context():
    from correction.local_llm import LocalLLM

    llm = LocalLLM.__new__(LocalLLM)
    llm.pipeline = None

    prompt = llm._build_prompt("hello world", "")
    assert "Context:" not in prompt
    assert "hello world" in prompt


@pytest.mark.asyncio
async def test_local_llm_extract_correction():
    from correction.local_llm import LocalLLM

    llm = LocalLLM.__new__(LocalLLM)
    llm.pipeline = None

    prompt = "Fix errors\nText: hellow\nCorrected:"
    generated = "Fix errors\nText: hellow\nCorrected: Hello"
    result = llm._extract_correction(generated, prompt)
    assert result == "Hello"


@pytest.mark.asyncio
async def test_local_llm_correct_empty_text():
    from correction.local_llm import LocalLLM

    llm = LocalLLM.__new__(LocalLLM)
    llm.pipeline = None

    with pytest.raises(RuntimeError):
        await llm.correct("", "")


@pytest.mark.asyncio
async def test_cloud_llm_no_api_key():
    from correction.cloud_llm import CloudLLM

    llm = CloudLLM.__new__(CloudLLM)
    llm.provider = "none"
    with pytest.raises(RuntimeError, match="No cloud API key"):
        await llm.correct("test text")


@pytest.mark.asyncio
async def test_cloud_llm_prompt_building():
    from correction.cloud_llm import CloudLLM

    llm = CloudLLM.__new__(CloudLLM)
    llm.provider = "none"
    prompt = llm._build_prompt("hello world", "context")
    assert "hello world" in prompt
    assert "context" in prompt
    assert "Fix any transcription errors" in prompt
