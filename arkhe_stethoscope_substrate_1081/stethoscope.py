import torch
import numpy as np
from typing import Any

class VectorTheosisStethoscope:
    """
    Substrate 1081: Transformer Canon / Stethoscope
    Hooks into PyTorch models to extract real hidden states during the forward pass
    and ingests them into the VectorTheosis OrchestratorRSI.
    """
    def __init__(self, orchestrator: Any, layer_idx: int = 6):
        self.orchestrator = orchestrator
        self.layer_idx = layer_idx
        self.hook_handle = None
        self.token_counter = 0

    def hook_fn(self, module, input, output):
        """
        The actual hook function that is called after the forward pass of the target layer.
        output is typically a tensor of shape (batch_size, seq_len, hidden_size).
        """
        # For simplicity, we assume batch_size=1 and process the last token's hidden state
        # in a real streaming scenario, or iterate over the sequence.

        # Output shape could be tuple or tensor depending on the transformer implementation
        if isinstance(output, tuple):
            hidden_states = output[0]
        else:
            hidden_states = output

        # Get the hidden state for the most recently processed token (last one in sequence)
        # Shape: (batch_size, seq_len, hidden_size) -> we want (hidden_size,)
        if len(hidden_states.shape) >= 3:
            latest_hidden_state = hidden_states[0, -1, :].detach().cpu().numpy()
        else:
            latest_hidden_state = hidden_states.detach().cpu().numpy().flatten()

        # Mock token text for now since we don't have the tokenizer here
        token_text = f"token_{self.token_counter}"

        # Ingest into Orchestrator RSI
        self.orchestrator.ingest_hidden_state(
            hidden_state=latest_hidden_state,
            token_text=token_text,
            token_id=self.token_counter
        )

        self.token_counter += 1

    def attach(self, model: torch.nn.Module, layer_name_or_module: Any = None):
        """
        Attaches the hook to a specific layer in the model.
        """
        target_module = None

        if layer_name_or_module is None:
            # Try to guess the layer based on common transformer architectures
            # (e.g., Hugging Face models)
            # Default to layer_idx (e.g., 6)
            try:
                # E.g., for BERT, LLaMA, etc.
                target_module = model.base_model.layers[self.layer_idx]
            except AttributeError:
                try:
                    target_module = model.transformer.h[self.layer_idx]
                except AttributeError:
                    try:
                        target_module = model.encoder.layer[self.layer_idx]
                    except AttributeError:
                        # Fallback: just attach to the model itself (not ideal)
                        target_module = model
        elif isinstance(layer_name_or_module, str):
            for name, module in model.named_modules():
                if name == layer_name_or_module:
                    target_module = module
                    break
        elif isinstance(layer_name_or_module, torch.nn.Module):
            target_module = layer_name_or_module

        if target_module is None:
            raise ValueError(f"Could not find target layer to attach hook.")

        self.hook_handle = target_module.register_forward_hook(self.hook_fn)
        print(f"Stethoscope attached to {target_module.__class__.__name__}")

    def detach(self):
        """Removes the hook."""
        if self.hook_handle is not None:
            self.hook_handle.remove()
            self.hook_handle = None
            print("Stethoscope detached.")
