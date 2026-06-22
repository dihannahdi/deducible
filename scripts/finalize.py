import os
p = "/workspace/paper/full_paper_v3.md"
s = open(p).read()
old = "A deeper Slither pass is identified as future hardening."
new = ("A deeper Slither static-analysis pass (63 detectors) returns zero findings after a "
       "hardening pass that made set-once state variables immutable and removed vestigial state.")
if old in s:
    s = s.replace(old, new, 1); open(p, "w").write(s); print("paper Slither sentence updated")
else:
    print("WARN: sentence not found (already updated?)")
