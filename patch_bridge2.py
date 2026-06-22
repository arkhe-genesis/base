with open("bridge/src/main.rs", "r") as f:
    code = f.read()

code = code.replace(
    "QueryProvenanceRequest, QueryProvenanceResponse,",
    "QueryProvenanceRequest, QueryProvenanceResponse, ZkVerifyRequest, ZkVerifyResponse, NostrPublishRequest, NostrPublishResponse,"
)
code = code.replace("blake3::hash", "sha2::Sha256::digest")
code = code.replace("use serde_json::Value;", "use serde_json::Value;\nuse sha2::Digest;")

with open("bridge/src/main.rs", "w") as f:
    f.write(code)
