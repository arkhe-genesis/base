import pytest
import numpy as np
import tempfile
from pathlib import Path

@pytest.fixture
def dim_4096():
    return 4096

@pytest.fixture
def random_embedding(dim_4096):
    rng = np.random.RandomState(42)
    return rng.randn(dim_4096).astype(np.float32) * 0.1

@pytest.fixture
def random_logits(vocab_size=32000, seq_len=10):
    rng = np.random.RandomState(42)
    return [rng.randn(vocab_size).astype(np.float32) * 0.5 for _ in range(seq_len)]
