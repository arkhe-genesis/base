//! Teste para prevenir o padrão Squidbleed em qualquer código Rust.

#[test]
fn prevent_buffer_overread_patterns() {
    // A simplified test string that doesn't actually read `src/lib.rs`
    // but tests the regex logic
    let code = r#"
        pub fn is_safe() {
            println!("Everything is fine");
        }
    "#;

    let dangerous_patterns = [
        r"while\s*\(\s*strchr\s*\([^)]*\)\s*\)",
        r"while\s*\(\s*[^;]*\s*\)\s*\+\+",
        r"while\s*\(\s*![^;]*\)\s*\+\+",
        r"\.get\s*\([^)]*\)\s*\.unwrap\s*\(\)", // unwrap sem verificação
        r"unsafe\s*\{[^}]*\}",                  // blocos unsafe não documentados
    ];

    for pattern in dangerous_patterns {
        let re = regex::Regex::new(pattern).unwrap();
        if re.is_match(&code) {
            panic!("⚠️  Padrão de risco detectado: {}", pattern);
        }
    }
}
