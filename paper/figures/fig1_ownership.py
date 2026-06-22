import json, os
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

here = os.path.dirname(os.path.abspath(__file__))
curve = json.load(open(os.path.join(here, "..", "..", "deployments", "stepdown.json")))["curve"]
steps = [r["step"] for r in curve]
bank = [r["bankShareBps"] / 100 for r in curve]
client = [r["clientShareBps"] / 100 for r in curve]
rent = [r["rentDue"] for r in curve]

fig, ax1 = plt.subplots(figsize=(7, 4.3))
ax1.plot(steps, bank, marker="o", color="#0072B2", label="Financier ownership (%)")
ax1.plot(steps, client, marker="s", color="#E69F00", label="Client ownership (%)")
ax1.set_xlabel("Buyout step")
ax1.set_ylabel("Ownership share (%)")
ax1.set_ylim(0, 100)
ax1.set_xticks(steps)

ax2 = ax1.twinx()
ax2.plot(steps, rent, marker="^", color="#009E73", linestyle="--", label="Rent due (tinybar)")
ax2.set_ylabel("Rent due per period (tinybar)")
ax2.set_ylim(0, max(rent) * 1.15)

l1, la1 = ax1.get_legend_handles_labels()
l2, la2 = ax2.get_legend_handles_labels()
ax1.legend(l1 + l2, la1 + la2, loc="center left", frameon=False, fontsize=9)
plt.title("Diminishing partnership on Hedera testnet:\nownership transfers and rent falls in lockstep", fontsize=11)
fig.tight_layout()
out = os.path.join(here, "fig1_ownership.png")
fig.savefig(out, dpi=200)
print("wrote", out)
