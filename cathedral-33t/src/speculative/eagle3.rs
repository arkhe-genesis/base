//! EAGLE-3 Speculative Decoding

use crate::tensor::Tensor;

pub struct Eagle3Decoder {
    draft_head: DraftHead,
    target_model: TargetModel,
    num_speculative_tokens: usize,
    disable_threshold_batch: usize,
    max_context_length: usize,
}

impl Eagle3Decoder {
    pub fn new(
        target_model: TargetModel,
        draft_head: DraftHead,
        num_tokens: usize,
        max_context_length: usize,
    ) -> Self {
        Self {
            draft_head,
            target_model,
            num_speculative_tokens: num_tokens,
            disable_threshold_batch: 32,
            max_context_length,
        }
    }

    pub fn generate(&self, prompt: &str, max_tokens: usize) -> String {
        let mut output = String::new();
        let mut tokens = self.tokenize(prompt);

        for _ in 0..max_tokens {
            // Verificar limite de contexto
            if tokens.len() >= self.max_context_length {
                break;
            }

            let draft_tokens = self.draft_head.generate(&tokens, self.num_speculative_tokens);
            let verified = self.target_model.verify(&tokens, &draft_tokens);

            // CORREÇÃO: limitar n_accepted ao tamanho de draft_tokens
            let n_accepted = verified.longest_valid_prefix().min(draft_tokens.len());

            // Verificar se adicionar tokens excede contexto
            if tokens.len() + n_accepted >= self.max_context_length {
                let remaining = self.max_context_length - tokens.len();
                for token in draft_tokens[..remaining.min(n_accepted)].iter() {
                    output.push_str(&self.detokenize(token));
                    tokens.push(*token);
                }
                break;
            }

            for token in draft_tokens[..n_accepted].iter() {
                output.push_str(&self.detokenize(token));
                tokens.push(*token);
            }

            if n_accepted < draft_tokens.len() {
                let correct_token = self.target_model.generate_next(&tokens);
                output.push_str(&self.detokenize(&correct_token));
                tokens.push(correct_token);
            }
        }

        output
    }

    fn tokenize(&self, text: &str) -> Vec<u32> {
        text.chars().map(|c| c as u32).collect()
    }

    fn detokenize(&self, token: &u32) -> String {
        std::char::from_u32(*token).unwrap_or('?').to_string()
    }
}

pub struct DraftHead {
    hidden_size: usize,
}

impl DraftHead {
    pub fn new(hidden_size: usize) -> Self {
        Self { hidden_size }
    }

    pub fn generate(&self, tokens: &[u32], k: usize) -> Vec<u32> {
        let mut result = Vec::with_capacity(k);
        for i in 0..k {
            let idx = (tokens.len() + i) % tokens.len();
            result.push(tokens[idx]);
        }
        result
    }
}

pub struct TargetModel {
    vocab_size: usize,
}

impl TargetModel {
    pub fn new(vocab_size: usize) -> Self {
        Self { vocab_size }
    }

    pub fn verify(&self, _prefix: &[u32], draft: &[u32]) -> VerificationResult {
        VerificationResult {
            accepted: draft.to_vec(),
        }
    }

    pub fn generate_next(&self, tokens: &[u32]) -> u32 {
        tokens.last().copied().unwrap_or(0)
    }
}

pub struct VerificationResult {
    accepted: Vec<u32>,
}

impl VerificationResult {
    pub fn longest_valid_prefix(&self) -> usize {
        self.accepted.len()
    }
}
