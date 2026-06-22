import os
here = "/workspace/paper"
sec = open(os.path.join(here, "sections.md")).read()
mr = open(os.path.join(here, "methodology_results.md")).read()
marker = "## 5. Discussion"
head, tail = sec.split(marker, 1)
tail = marker + tail               # sections 5 + 6
mr_body = mr[mr.index("## 3."):]   # sections 3 + 4 (drop the H1 + note)
full = head.rstrip() + "\n\n" + mr_body.rstrip() + "\n\n" + tail.rstrip() + "\n"
open(os.path.join(here, "full_paper.md"), "w").write(full)
print("wrote full_paper.md, chars:", len(full))
