with open("bridge/src/main.rs", "r") as f:
    code = f.read()

code = code.replace(".as_bytes().to_vec()", ".to_vec()")

with open("bridge/src/main.rs", "w") as f:
    f.write(code)
