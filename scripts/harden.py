import os
C = "/workspace/contracts"

def patch(fn, subs):
    p = os.path.join(C, fn)
    s = open(p).read()
    for old, new in subs:
        assert old in s, "NOT FOUND in " + fn + ": " + repr(old)
        s = s.replace(old, new, 1)
    open(p, "w").write(s)
    print("patched", fn)

patch("MusharakahMutanaqisah.sol", [
    ("uint256 public rentPerPeriodPerBps;", "uint256 public immutable rentPerPeriodPerBps;"),
])
patch("MusharakahMutanaqisahV2.sol", [
    ("uint256 public rentPerPeriodPerBps;", "uint256 public immutable rentPerPeriodPerBps;"),
])
patch("MusharakahMutanaqisahV3.sol", [
    ("address public token;", "address public immutable token;"),
    ("uint64 public totalUnits;", "uint64 public immutable totalUnits;"),
    ("uint256 public rentPerPeriodPerUnit;", "uint256 public immutable rentPerPeriodPerUnit;"),
])
patch("MusharakahMutanaqisahV4.sol", [
    ("uint256 public rentPerPeriodPerBps;", "uint256 public immutable rentPerPeriodPerBps;"),
    ("    bool public settled;\n", ""),
    ('require(active && !settled && !rescinded, "not live")', 'require(active && !rescinded, "not live")'),
])
print("done")
