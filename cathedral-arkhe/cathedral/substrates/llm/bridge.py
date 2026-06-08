import time
import numpy as np
from pathlib import Path
from cathedral.substrates.gguf.bridge import GGUFBridgeV3


class LlamaCppBridgeV3:
    def __init__(self, model_path=None, n_ctx=2048, n_gpu_layers=-1, verbose=False):
        self.model_path = model_path; self.n_ctx = n_ctx
        self.n_gpu_layers = n_gpu_layers; self.verbose = verbose
        self._llm = None; self._gguf = None; self._inference_log = []
        self._available = False; self._vocab_size = 0; self._n_embd = 0

    def _check_available(self):
        try:
            import llama_cpp; self._available = True; return True
        except ImportError: self._available = False; return False

    def load(self, model_path=None):
        if not self._check_available(): return False
        from llama_cpp import Llama
        path = model_path or self.model_path
        if not path or not Path(path).exists(): return False
        self.model_path = path
        try:
            self._llm = Llama(model_path=path, n_ctx=self.n_ctx,
                              n_gpu_layers=self.n_gpu_layers, verbose=self.verbose,
                              embedding=True, logits_all=True)
            self._n_embd = self._llm.n_embd(); self._vocab_size = self._llm.n_vocab()
            self._gguf = GGUFBridgeV3(); self._gguf.open(path)
            return True
        except Exception as e:
            print(f"[LlamaCppBridgeV3] Error: {e}"); return False

    def generate_with_full_extraction(self, prompt, max_tokens=50, temperature=0.7, top_p=0.9):
        if not self._llm: return {"status": "MODEL_NOT_LOADED"}
        start = time.time()
        tokens_in = self._llm.tokenize(prompt.encode("utf-8"))
        output = self._llm(prompt, max_tokens=max_tokens, temperature=temperature,
                           top_p=top_p, logits_all=True, echo=True)
        gen_time = time.time() - start

        logits_per_position = []
        if hasattr(output, "logits") and output.logits is not None:
            raw_logits = np.array(output.logits, dtype=np.float32)
            for i in range(raw_logits.shape[0]): logits_per_position.append(raw_logits[i])
        elif "logits" in output and output["logits"] is not None:
            raw_logits = np.array(output["logits"], dtype=np.float32)
            for i in range(raw_logits.shape[0]): logits_per_position.append(raw_logits[i])

        entropy_per_position = []
        for logits in logits_per_position:
            probs = np.exp(logits - np.max(logits))
            probs = probs / (np.sum(probs) + 1e-12)
            log_probs = np.log(probs + 1e-12)
            entropy = float(-np.sum(probs * log_probs))
            entropy_per_position.append(round(entropy, 4))

        embeddings = {}; emb_vec = None
        try:
            emb_result = self._llm.create_embedding([prompt])
            if emb_result.get("data"):
                emb_vec = np.array(emb_result["data"][0]["embedding"], dtype=np.float32)
                embeddings["mean"] = emb_vec.tolist()
        except Exception as e: embeddings["error"] = str(e)

        return {"status": "SUCCESS", "prompt": prompt, "prompt_tokens": len(tokens_in),
                "generated_text": output.get("choices", [{}])[0].get("text", ""),
                "generated_tokens": output.get("usage", {}).get("completion_tokens", 0),
                "generation_time": round(gen_time, 4),
                "logits_per_position": [l.tolist() for l in logits_per_position[:64]],
                "logits_shape": [len(logits_per_position), len(logits_per_position[0]) if logits_per_position else 0],
                "entropy_per_position": entropy_per_position,
                "embeddings": embeddings, "embedding_array": emb_vec,
                "n_embd": self._n_embd, "vocab_size": self._vocab_size}

    def get_telemetry(self):
        return {"module": "LlamaCppBridgeV3", "version": "3.0.0", "substrate": "1094.2",
                "seal": "LLAMA-CPP-BRIDGE-1094.2-v3.0.0-2026-06-07",
                "llama_cpp_available": self._available, "model_loaded": self._llm is not None,
                "n_embd": self._n_embd, "vocab_size": self._vocab_size,
                "inference_runs": len(self._inference_log)}
