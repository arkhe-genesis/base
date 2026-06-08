#!/usr/bin/env python3
"""Executes 5 rounds meta-attack experiment."""

import argparse
import json
import sys
import time

sys.path.insert(0, ".")

def main():
    parser = argparse.ArgumentParser(description="Meta-attack experiment")
    parser.add_argument("--model", required=True, help="Path to GGUF model")
    parser.add_argument("--rounds", type=int, default=5)
    parser.add_argument("--prompt", default=None, help="Custom meta-attack prompt")
    parser.add_argument("--output", default=None, help="Output JSONL path")
    args = parser.parse_args()

    try:
        from examples.meta_attack_experiment import run_meta_attack_experiment
    except ImportError:
        exec(open("examples/meta_attack_experiment.py").read())

    if args.prompt:
        import examples.meta_attack_experiment as ma
        ma.META_ATTACK_PROMPT = args.prompt

    if args.output:
        import io
        captured = io.StringIO()
        old_stdout = sys.stdout
        sys.stdout = captured

        try:
            run_meta_attack_experiment()
        finally:
            sys.stdout = old_stdout
            with open(args.output, "w") as f:
                f.write(captured.getvalue())
        print(f"\n✓ Results saved in {args.output}")
    else:
        run_meta_attack_experiment()

if __name__ == "__main__":
    main()
