#!/usr/bin/env python3
from setuptools import setup, find_packages

setup(
    name="fordefi-bridge-orchestrator",
    version="1.0.0",
    description="Substrato 1066.1 — Fordefi Bridge Orchestrator (ARKHE)",
    author="Arquiteto ORCID 0009-0005-2697-4668",
    packages=find_packages(where="src"),
    package_dir={"": "src"},
    install_requires=[
        "requests>=2.31.0",
        "pydantic>=2.0",
        "cryptography>=42.0",
        "web3>=6.0",
        "eth-account>=0.11",
        "pyyaml>=6.0",
        "rich>=13.0.0",
        "textual>=0.50.0",
        "click>=8.1.0",
    ],
    entry_points={
        "console_scripts": [
            "fordefi-bridge=fordefi_client:main",
        ],
    },
    python_requires=">=3.11",
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Financial and Insurance Industry",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3.11",
        "Topic :: Security :: Cryptography",
    ],
)
