# 🏛️ Arkhe OS — Constitution of Secure-by-Construction

**Versão:** 1.0.0
**Data:** 2026-06-25
**Status:** ✅ RATIFICADA

---

## 📋 Princípios Não Negociáveis

### 1. Segurança de Memória por Construção

- Todo bloco `unsafe` DEVE ter uma prova Kani correspondente.
- A prova DEVE cobrir **todas as entradas possíveis**, não apenas casos de teste.
- Exceção: blocos `unsafe` em crates de terceiros (com exceção documentada).

**Fonte:** CWE-119 (Buffer Overflow), Squidbleed (CVE-2026-47729)

### 2. Verificação Formal como Gate de CI

- TLA+ DEVE passar no typechecking e no model checking do Apalache.
- Kani DEVE passar em todas as provas definidas.
- CI NÃO PODE ser aprovado se qualquer verificação falhar.

**Fonte:** NIST SSDF, OWASP SAMM

### 3. Criptografia Pós-Quântica como Baseline

- ML-DSA-65 + Ed25519 híbrido DEVE ser usado para assinaturas.
- ML-KEM-1024 DEVE ser usado para estabelecimento de chaves.
- SHA-3 DEVE ser usado para hashing (BLAKE3 como alternativa acelerada).

**Fonte:** NIST FIPS 203, FIPS 204, Executive Order 14028

### 4. Auditoria Contínua e Imutável

- Cada ação DEVE ser registrada no WAL com assinatura híbrida.
- O WormGraph DEVE armazenar hashes imutáveis das ações.
- Auditoria NUNCA DEVE ser mutável após o commit.

**Fonte:** NIS2 (UE), Cyber Resilience Act (UE)

### 5. Revisão Adversária Obrigatória

- Cada PR DEVE ser revisado por um agente adversário (metodologia Cloudflare).
- O agente DEVE tentar ativamente **refutar** a segurança da mudança.
- O autor NÃO PODE ser o revisor do próprio PR.

**Fonte:** Metodologia Cloudflare Security Audit Skill

### 6. Política de Olhos Frescos (Fresh Eyes)

- Nenhum crate DEVE ser mantido pela mesma equipe por mais de 3 anos.
- Transição DEVE ocorrer em até 30 dias.
- Documentação DEVE ser atualizada durante a transição.

**Fonte:** Squidbleed (CVE-2026-47729) — viés de familiaridade

### 7. Política de Descontinuação (Squidbleed Sunset)

- Código com mais de 5 anos DEVE passar por revisão completa.
- Revisão DEVE incluir análise de padrões históricos (ex: strchr).
- Código não revisado DEVE ser descontinuado.

**Fonte:** Squidbleed (CVE-2026-47729) — dívida técnica

---

## 🔗 Rastreabilidade

| Princípio | Requisito Técnico | Verificação |
|-----------|-------------------|-------------|
| 1. Memória | Kani proofs para cada `unsafe` | CI gate |
| 2. Formal | Apalache + TLA+ | CI gate |
| 3. PQC | ML-DSA-65 + ML-KEM-1024 | `SignatureClass` check |
| 4. Auditoria | WAL + WormGraph | `ActionCompute` trait |
| 5. Revisão | AdversarialReviewer | PR check |
| 6. Fresh Eyes | OWNERS_ROTATION.yaml | Semanal |
| 7. Sunset | legacy_sunset.rs | Mensal |

---

## 📝 Emendas

| Data | Versão | Descrição |
|------|--------|-----------|
| 2026-06-25 | 1.0.0 | Ratificação inicial |
