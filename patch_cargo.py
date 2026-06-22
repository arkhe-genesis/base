with open("Cargo.toml", "r") as f:
    code = f.read()

code = code.replace(
    "[workspace.dependencies]",
    "[workspace.dependencies]\n# Ferramentas de desenvolvimento (globais)\ncargo-insta = \"1.40\"\ncargo-llvm-cov = \"0.5\"\ncargo-deadlinks = \"0.9\"\ncargo-sbom = \"0.4\"\n\n# Para o xtask\ncolored = \"2.1\"\nwhich = \"6.0\""
)

with open("Cargo.toml", "w") as f:
    f.write(code)
