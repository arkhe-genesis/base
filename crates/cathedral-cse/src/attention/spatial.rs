use ndarray::{s, Array2, ArrayView2}; // ✅ CSE-CRIT-002
use rayon::prelude::*; // ✅ CSE-CRIT-001

#[derive(Debug, Clone)]
pub struct KVPartition {
    pub keys: Array2<f32>,
    pub values: Array2<f32>,
    pub block_size: usize,
    pub position: usize,
}

pub struct SpatialAttentionEngine {
    block_size: usize,
    num_blocks: usize,
    partitions: Vec<KVPartition>,
    pub temperature: f32,
    use_sparse: bool,
}

impl SpatialAttentionEngine {
    pub fn new(block_size: usize, num_blocks: usize, temperature: f32) -> Self {
        Self {
            block_size,
            num_blocks,
            partitions: Vec::with_capacity(num_blocks),
            temperature,
            use_sparse: true,
        }
    }

    pub fn index_context(&mut self, keys: Array2<f32>, values: Array2<f32>) {
        let total_len = keys.shape()[0];
        let num_blocks = total_len.div_ceil(self.block_size);
        self.partitions.clear();
        for i in 0..num_blocks {
            let start = i * self.block_size;
            let end = (start + self.block_size).min(total_len);
            let key_block = keys.slice(s![start..end, ..]).to_owned();
            let value_block = values.slice(s![start..end, ..]).to_owned();
            self.partitions.push(KVPartition {
                keys: key_block,
                values: value_block,
                block_size: end - start,
                position: start,
            });
        }
    }

    pub fn sparse_attention(&self, query: ArrayView2<f32>) -> Array2<f32> {
        if !self.use_sparse || self.partitions.is_empty() {
            return self.dense_attention(query);
        }

        let mut scores: Vec<_> = self
            .partitions
            .par_iter()
            .enumerate()
            .map(|(idx, p)| (idx, self.compute_block_similarity(&query, &p.keys)))
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let k = (self.num_blocks.min(self.partitions.len()) as f32 * 0.3) as usize;
        let selected: Vec<_> = scores.into_iter().take(k).collect();

        // ✅ CSE-CRIT-003: proteção contra partitions vazia
        let value_dim = self
            .partitions
            .first()
            .map(|p| p.values.shape()[1])
            .unwrap_or(128);
        let mut output = Array2::<f32>::zeros((query.shape()[0], value_dim));

        for (idx, _) in selected {
            let p = &self.partitions[idx];
            let attn = self.dense_attention_on_block(&query, &p.keys, &p.values);
            output = output + attn;
        }

        output
    }

    fn dense_attention(&self, query: ArrayView2<f32>) -> Array2<f32> {
        Array2::<f32>::zeros((query.shape()[0], 128))
    }

    fn dense_attention_on_block(
        &self,
        query: &ArrayView2<f32>,
        _keys: &Array2<f32>,
        values: &Array2<f32>,
    ) -> Array2<f32> {
        Array2::<f32>::zeros((query.shape()[0], values.shape()[1]))
    }

    // ✅ CSE-HIGH-005: norma L2
    fn compute_block_similarity(&self, query: &ArrayView2<f32>, keys: &Array2<f32>) -> f32 {
        let q_norm = query.mapv(|x| x * x).sum().sqrt();
        let k_norm = keys.mapv(|x| x * x).sum().sqrt();

        let mut dot = 0.0;
        let r = query.shape()[0].min(keys.shape()[0]);
        let c = query.shape()[1].min(keys.shape()[1]);

        for i in 0..r {
            for j in 0..c {
                dot += query[[i, j]] * keys[[i, j]];
            }
        }

        if q_norm > 0.0 && k_norm > 0.0 {
            dot / (q_norm * k_norm)
        } else {
            0.0
        }
    }

    pub fn compress_kv_cache(&mut self, compression_ratio: f32) {
        let target = (self.partitions.len() as f32 * compression_ratio) as usize;
        if target < self.partitions.len() {
            let mut scores: Vec<_> = self
                .partitions
                .iter()
                .enumerate()
                .map(|(i, p)| (i, p.keys.mapv(|x| x * x).sum().sqrt()))
                .collect();
            scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            let keep: Vec<usize> = scores.into_iter().take(target).map(|(i, _)| i).collect();
            let new_partitions = keep
                .iter()
                .map(|&i| self.partitions[i].clone())
                .collect();
            self.partitions = new_partitions;
        }
    }
}
