import re

def translate_file(filepath, replacements):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    for pt, en in replacements.items():
        content = content.replace(pt, en)

    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(content)

# zkcbdc_engine.py replacements
engine_replacements = {
    'zkCBDC — Substrato 1010': 'zkCBDC — Substrate 1010',
    'Motor completo de validação com ZK-SNARKs, Nullifiers, Passport Gateway e TemporalChain.': 'Complete validation engine with ZK-SNARKs, Nullifiers, Passport Gateway, and TemporalChain.',
    'Arquiteto ORCID': 'Architect ORCID',
    'Constantes canônicas': 'Canonical constants',
    'Estado de uma conta no livro-razão confidencial.': 'Account state in the confidential ledger.',
    'Com(saldo, r) — Pedersen': 'Com(balance, r) — Pedersen',
    'não verificado': 'unverified',
    'básico': 'basic',
    'completo': 'complete',
    'Transação confidencial com prova ZK.': 'Confidential transaction with ZK proof.',
    'Compromissos (Pedersen Commitments)': 'Commitments (Pedersen Commitments)',
    'impede gasto duplo sem vincular transações': 'prevents double spending without linking transactions',
    'simulada; em produção: Groth16/Plonk sobre curva BN254': 'simulated; in production: Groth16/Plonk over BN254 curve',
    'Metadados': 'Metadata',
    'Motor de validação da zkCBDC.': 'zkCBDC validation engine.',
    'Héstia guarda o lar (privacidade);': 'Hestia guards the home (privacy);',
    'Hermes comercia (transações);': 'Hermes trades (transactions);',
    'Themis julga em segredo (ZK-proofs).': 'Themis judges in secret (ZK-proofs).',
    'Volume total em centavos (auditável publicamente)': 'Total volume in cents (publicly auditable)',
    'Cria uma conta com saldo inicial.': 'Creates an account with an initial balance.',
    'Adiciona conta à lista de sanções.': 'Adds an account to the sanctions list.',
    'Congela uma conta (ex: ordem judicial com prova).': 'Freezes an account (e.g., court order with proof).',
    'Cria uma transação confidencial com todas as verificações.': 'Creates a confidential transaction with all checks.',
    'Verificações básicas': 'Basic checks',
    'Nullifier para prevenir gasto duplo': 'Nullifier to prevent double spending',
    'Em ZK, o nullifier é derivado do segredo (chave) e do id da transação para ser determinístico': 'In ZK, the nullifier is derived from the secret (key) and the transaction id to be deterministic',
    'Compromissos criptográficos': 'Cryptographic commitments',
    'ZK-Proof (simulada)': 'ZK-Proof (simulated)',
    'Verificações Axiarchy (954)': 'Axiarchy checks (954)',
    'Registrar': 'Register',
    'Prova de preservação da oferta monetária': 'Money supply preservation proof',
    'Simular ancoragem na TemporalChain (923)': 'Simulate anchoring on TemporalChain (923)',
    'Verifica a prova ZK de uma transação.': 'Verifies the ZK proof of a transaction.',
    'Audita a oferta monetária sem revelar transações individuais.': 'Audits the money supply without revealing individual transactions.',
    'Nenhum valor individual foi exposto. Privacidade preservada.': 'No individual value was exposed. Privacy preserved.',
    'Relatório canônico.': 'Canonical report.',
    'OFERTA MONETÁRIA': 'MONEY SUPPLY',
    'TRANSAÇÕES': 'TRANSACTIONS',
    'VOLUME TOTAL': 'TOTAL VOLUME',
    'PROVAS DE CUNHAGEM': 'MINT PROOFS',
    'CONTAS': 'ACCOUNTS',
    'CONGELADAS': 'FROZEN',
    'EM SANÇÕES': 'SANCTIONED',
    'INVARIANTE': 'INVARIANT',
    'PRINCÍPIOS AXIARCHY (954)': 'AXIARCHY PRINCIPLES (954)',
    'P1 - Diagnóstico: Oferta monetária verificável sem exposição': 'P1 - Diagnostics: Verifiable money supply without exposure',
    'P2 - Intervenção Mínima: Apenas nullifiers são públicos': 'P2 - Minimal Intervention: Only nullifiers are public',
    'P3 - Soberania: Cidadãos controlam suas chaves privadas': 'P3 - Sovereignty: Citizens control their private keys',
    'P4 - Transparência: Provas ZK são publicamente verificáveis': 'P4 - Transparency: ZK proofs are publicly verifiable',
    'P5 - Descentralização: Livro-razão distribuído via TemporalChain': 'P5 - Decentralization: Distributed ledger via TemporalChain',
    'P6 - Consentimento: KYC opt-in via Passport Gateway': 'P6 - Consent: Opt-in KYC via Passport Gateway',
    'P7 - Proporcionalidade: Congelamento seletivo, nunca confisco geral': 'P7 - Proportionality: Selective freezing, never general confiscation',
    'Demonstração': 'Demonstration',
    'Criar contas': 'Create accounts',
    'Transação normal': 'Normal transaction',
    'Tentativa de gasto duplo': 'Double spend attempt',
    'ALERTA: Gasto duplo não detectado!': 'WARNING: Double spend not detected!',
    '✓ Gasto duplo detectado': '✓ Double spend detected',
    'Sanções': 'Sanctions',
    'rejeitada por sanções': 'rejected due to sanctions',
    'ODÔMETRO': 'ODOMETER'
}
translate_file('arkhe_zkcbdc_substrate_1010/zkcbdc_engine.py', engine_replacements)

# zkcbdc_schema.yaml replacements
schema_replacements = {
    'SUBSTRATO 1010': 'SUBSTRATE 1010',
    'Schema Canônico YAML': 'Canonical YAML Schema',
    'Arquiteto ORCID': 'Architect ORCID',
    'substrato': 'substrate',
    'nome': 'name',
    'tipo': 'type',
    'Infraestrutura Financeira / Privacidade / ZK-Proofs': 'Financial Infrastructure / Privacy / ZK-Proofs',
    'metadados': 'metadata',
    'timestamp_canonizacao': 'canonization_timestamp',
    'arquiteto_orcid': 'architect_orcid',
    'deidades': 'deities',
    'Héstia': 'Hestia',
    'Lar e privacidade': 'Home and privacy',
    'Hermes': 'Hermes',
    'Comércio': 'Commerce',
    'Themis': 'Themis',
    'Justiça e lei': 'Justice and law',
    'linguagens': 'languages',
    'papel': 'role',
    'Criptografia pós-quântica para os compromissos Pedersen': 'Post-quantum cryptography for Pedersen commitments',
    'Validação ética (P1-P7) de cada transação': 'Ethical validation (P1-P7) of each transaction',
    'Ancoragem imutável de nullifiers e mint proofs': 'Immutable anchoring of nullifiers and mint proofs',
    'Verificação AML/KYC sem exposição de dados': 'AML/KYC verification without data exposure',
    'Alterações na política monetária exigem consenso': 'Changes in monetary policy require consensus',
    'Prova de humanidade para KYC (zk-SANCTIONS)': 'Proof of humanity for KYC (zk-SANCTIONS)',
    'criptografia': 'cryptography',
    'curva_eliptica': 'elliptic_curve',
    'ou': 'or',
    'otimizado para ZK': 'optimized for ZK',
    'compromissos': 'commitments',
    'impede gasto duplo': 'prevents double spending',
    'invariantes': 'invariants',
    'preservação da oferta': 'supply preservation',
    'único por transação': 'unique per transaction',
    'válido': 'valid',
    'testes': 'tests',
    'suites': 'suites'
}
translate_file('arkhe_zkcbdc_substrate_1010/zkcbdc_schema.yaml', schema_replacements)

# decree_1010.md replacements
decree_replacements = {
    'Decreto Canônico — Substrato 1010': 'Canonical Decree — Substrate 1010',
    'Propósito': 'Purpose',
    'Data': 'Date',
    'Arquiteto': 'Architect',
    'Preâmbulo': 'Preamble',
    'O dinheiro digital de banco central não precisa ser um panóptico.': 'Central bank digital money doesn\'t have to be a panopticon.',
    'A Catedral ARKHE, fiel aos princípios da Axiarquia (954), institui o Substrato 1010 — zkCBDC — como a infraestrutura de referência para uma moeda digital que preserva a privacidade do cidadão através de provas de conhecimento zero.': 'The ARKHE Cathedral, faithful to the principles of Axiarchy (954), institutes Substrate 1010 — zkCBDC — as the reference infrastructure for a digital currency that preserves citizen privacy through zero-knowledge proofs.',
    'O trilema fundamental das CBDCs — privacidade do usuário, prevenção de ilícitos e controle da oferta monetária — é resolvido pela matemática: ZK-SNARKs permitem provar a validade de uma transação sem revelar remetente, destinatário ou valor.': 'The fundamental trilemma of CBDCs — user privacy, prevention of illicit acts, and control of the money supply — is solved by mathematics: ZK-SNARKs allow proving the validity of a transaction without revealing sender, recipient, or value.',
    'A Catedral não emite moeda. Ela VALIDA as provas.': 'The Cathedral does not issue currency. It VALIDATES the proofs.',
    'Deidades Patronas': 'Patron Deities',
    'Deidade': 'Deity',
    'Domínio': 'Domain',
    'Função': 'Function',
    'Héstia': 'Hestia',
    'Lar e privacidade': 'Home and privacy',
    'Guarda os segredos financeiros dos cidadãos': 'Guards citizens\' financial secrets',
    'Comércio': 'Commerce',
    'Facilita as transações': 'Facilitates transactions',
    'Justiça e lei': 'Justice and law',
    'Julga a validade das provas em segredo': 'Judges the validity of proofs in secret',
    'Arquitetura': 'Architecture',
    'Componente': 'Component',
    'Tecnologia': 'Technology',
    'Manifesto': 'Manifesto',
    'Com ZK-SNARKs, cada cidadão pode PROVAR que pagou seus impostos, que não lava dinheiro, e que a oferta monetária está intacta — SEM REVELAR seu saldo, seu histórico de compras, ou suas contrapartes.': 'With ZK-SNARKs, every citizen can PROVE that they paid their taxes, that they do not launder money, and that the money supply is intact — WITHOUT REVEALING their balance, their purchase history, or their counterparties.',
    'Odômetro': 'Odometer'
}
translate_file('arkhe_zkcbdc_substrate_1010/decree_1010.md', decree_replacements)

# tests/test_zkcbdc.py replacements
test_replacements = {
    'Testes canônicos — Substrato 1010': 'Canonical tests — Substrate 1010'
}
translate_file('arkhe_zkcbdc_substrate_1010/tests/test_zkcbdc.py', test_replacements)

print("Translation completed.")
