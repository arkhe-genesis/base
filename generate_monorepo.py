import os

OUTPUT_FILE = "arkhe-os-monorepo.txt"
BASE_DIR = "arkhe-os"
EXCLUDE_DIRS = [".git", "target"]

def main():
    with open(OUTPUT_FILE, "w") as out:
        out.write("# Arkhe OS Monorepo Complete\n\n")

        for root, dirs, files in os.walk(BASE_DIR):
            dirs[:] = [d for d in dirs if d not in EXCLUDE_DIRS]

            for file in sorted(files):
                if file == ".DS_Store" or file.endswith(".pyc") or file == "scaffold.py":
                    continue

                file_path = os.path.join(root, file)
                rel_path = os.path.relpath(file_path, BASE_DIR)

                try:
                    with open(file_path, "r") as f:
                        content = f.read()

                    ext = os.path.splitext(file)[1][1:]
                    if ext == "":
                        ext = "text"
                    if ext == "rs":
                        ext = "rust"

                    out.write(f"## `{rel_path}`\n\n")
                    out.write(f"```{ext}\n")
                    out.write(content)
                    if not content.endswith("\n"):
                        out.write("\n")
                    out.write("```\n\n")

                except UnicodeDecodeError:
                    pass

if __name__ == "__main__":
    main()
