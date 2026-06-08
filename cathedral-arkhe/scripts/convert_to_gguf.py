#!/usr/bin/env python3
"""Converts HuggingFace model to GGUF with quantization."""

import argparse
import subprocess
import sys

def main():
    parser = argparse.ArgumentParser(description="HF → GGUF conversion")
    parser.add_argument("model_id", help="HuggingFace model ID or local path")
    parser.add_argument("--quant", default="q4_k_m",
                        choices=["q4_0", "q4_1", "q5_0", "q5_1", "q4_k_s", "q4_k_m",
                                 "q5_k_s", "q5_k_m", "q6_k", "q8_0"],
                        help="Quantization (default: q4_k_m)")
    parser.add_argument("--out", default=None, help="Output path (default: auto)")
    parser.add_argument("--ctx", type=int, default=4096, help="Context length (default: 4086→4096)")
    args = parser.parse_args()

    model_id = args.model_id
    out_name = args.out or f"{model_id.split('/')[-1]}-{args.quant}.gguf"

    cmd = [
        sys.executable, "-m", "llama_cpp.convert",
        model_id,
        "--outtype", "gguf",
        "--outfile", out_name,
        "--outquant", args.quant,
    ]

    print(f"Converting: {model_id} → {out_name} ({args.quant})")
    print(f"Command: {' '.join(cmd)}")

    result = subprocess.run(cmd, capture_output=True, text=True)
    print(result.stdout)
    if result.returncode != 0:
        print(f"Error: {result.stderr}")
        sys.exit(1)

    print(f"\n✓ Model saved: {out_name}")
    print(f"  Size: {__import__('os').path.getsize(out_name) / 1e9:.2f} GB")

if __name__ == "__main__":
    main()
