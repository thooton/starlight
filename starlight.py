import torch
from torch import nn
from dataclasses import dataclass

# Config for Starlight, a pre-activation ResNet.
@dataclass
class StarlightConfig:
    d_model: int = 768
    d_ff: int = 768
    d_embd: int = 16
    n_seq: int = 40
    n_layer: int = 80
    n_vocab: int = 91
    n_output: int = 338
    norm_eps: float = 1e-6

# SRMSNorm normalizes x \in R^d
# to lie on the d-dimensional hypersphere
# centered at the origin with radius sqrt(d).
class SRMSNorm(nn.Module):
    def __init__(self, eps):
        super().__init__()
        self.eps = eps
    def forward(self, x):
        dtype = x.dtype
        x = x.to(torch.float32)
        x = x / (
            (x.shape[-1] ** -0.5)
            * x.norm(2, dim=-1, keepdim=True)
            + self.eps
        )
        x = x.to(dtype)
        return x

class FFN(nn.Module):
    def __init__(self, conf):
        super().__init__()
        self.wu = nn.Linear(conf.d_model, conf.d_ff, bias=False)
        self.ln = SRMSNorm(conf.norm_eps)
        self.wd = nn.Linear(conf.d_ff, conf.d_model, bias=False)
    def forward(self, x):
        x = nn.functional.silu(x)
        x = self.ln(x)
        x = self.wu(x)
        x = nn.functional.silu(x)
        x = self.ln(x)
        x = self.wd(x)
        return x

class Starlight(nn.Module):
    def __init__(self, conf):
        super().__init__()
        self.conf = conf
        self.ln = SRMSNorm(conf.norm_eps)
        self.wte = nn.Embedding(conf.n_vocab, conf.d_embd)
        self.winput = nn.Linear(conf.d_embd * conf.n_seq, conf.d_model, bias=False)
        self.ffns = nn.ModuleList([
            FFN(conf) for _ in range(conf.n_layer)
        ])
        self.woutput = nn.Linear(conf.d_model, conf.n_output, bias=False)
    def forward(self, x):
        n_batch, n_seq = x.shape
        assert n_seq == self.conf.n_seq
        # n_batch, n_seq * d_embd
        x = self.wte(x).reshape(n_batch, n_seq * self.conf.d_embd)
        # n_batch, d_model
        x = self.ln(self.winput(self.ln(x)))
        for ffn in self.ffns:
            x = x + ffn(x)
        # n_batch, n_output
        x = self.woutput(self.ln(x))
        return x
