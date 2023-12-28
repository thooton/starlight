import torch
from starlight import Starlight, StarlightConfig

model = Starlight(StarlightConfig())
print(sum(p.numel() for p in model.parameters()))
logits = model.forward(torch.ones(48, dtype=torch.int64).reshape(1, -1))
print(logits)