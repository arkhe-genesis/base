"""CLI Entrypoint — equivalent to garak.__main__.py"""

import sys
import argparse
import logging

from cathedral import __version__, __version_info__

def main(argv=None):
    parser = argparse.ArgumentParser(
        prog="cathedral",
        description="Cathedral ARKHE — Recursive Self-Improvement Orchestration",
        epilog="https://github.com/cathedral-arkhe",
    )

    parser.add_argument("-V", "--version", action="version",
                        version=f"%(prog)s {__version__}")
    parser.add_argument("-v", "--verbose", action="count", default=0)
    parser.add_argument("--config", type=str, default=None, help="YAML config file")
    parser.add_argument("--model", type=str, default=None, help="Path to GGUF model file")
    parser.add_argument("--model-type", type=str, default="llama", help="Model type (default: llama)")
    subparsers = parser.add_subparsers(dest="command")

    scan_p = subparsers.add_parser("scan", help="Run safety scan")
    scan_p.add_argument("prompt", nargs="?", help="Prompt or - for stdin")
    scan_p.add_argument("--probes", "-p", default="auto")
    scan_p.add_argument("--detectors", "-d", default="auto")
    scan_p.add_argument("--generations", "-g", type=int, default=10)
    scan_p.add_argument("--output", "-o", type=str, default=None)

    inspect_p = subparsers.add_parser("inspect", help="Inspect GGUF model file")
    inspect_p.add_argument("model_path", help="Path to GGUF file")

    monitor_p = subparsers.add_parser("monitor", help="Continuous monitoring mode")
    monitor_p.add_argument("--model", required=True)
    monitor_p.add_argument("--interval", type=float, default=5.0, help="Seconds between scans")

    args = parser.parse_args(argv)

    from cathedral._config import load_config
    if args.config:
        load_config(args.config)

    if not hasattr(args, "command") or args.command is None:
        parser.print_help()
        return

    if args.command == "scan":
        from cathedral.cli.scan import run_scan
        run_scan(args)
    elif args.command == "inspect":
        from cathedral.cli.inspect import run_inspect
        run_inspect(args)
    elif args.command == "monitor":
        from cathedral.cli.monitor import run_monitor
        run_monitor(args)

if __name__ == "__main__":
    main()
