import json, os
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

here = os.path.dirname(os.path.abspath(__file__))
curve = json.load(open(os.path.join(here, "..", "..", "deployments", "stepdown.json")))["curve"]

# A representative 10,000,000-tinybar write-down, apportioned by the contract's own
# loss rule (Invariant I3) at each ownership ratio reached during the step-down.
# The 60/40 stage matches the real on-chain syncValuation event (6,000,000 / 4,000,000).
L = 10_000_000
bank_borne = [L * r["bankShareBps"] / 10000 for r in curve]
client_borne = [L * r["clientShareBps"] / 10000 for r in curve]
labels = [f"step {r['step']}\n{r['bankShareBps']//100}/{r['clientShareBps']//100}%" for r in curve]

x = np.arange(len(curve))
fig, ax = plt.subplots(figsize=(7, 4.3))
ax.bar(x, bank_borne, color="#0072B2", label="Borne by financier")
ax.bar(x, client_borne, bottom=bank_borne, color="#E69F00", label="Borne by client")
ax.set_xticks(x)
ax.set_xticklabels(labels, fontsize=8)
ax.set_ylabel("Loss borne (tinybar)")
ax.set_xlabel("Ownership ratio (financier / client)")
ax.set_title("Proportional loss-sharing of a 10,000,000-tinybar write-down\nby live ownership ratio (the financier cannot exit whole)", fontsize=11)
ax.legend(frameon=False, fontsize=9)
ax.annotate("on-chain measured\n(6.0M / 4.0M)", xy=(1, L), xytext=(1.6, L * 1.02),
            fontsize=8, arrowprops=dict(arrowstyle="->", color="#555555"))
fig.tight_layout()
out = os.path.join(here, "fig2_loss.png")
fig.savefig(out, dpi=200)
print("wrote", out)
