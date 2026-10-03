#!/usr/bin/env python3
from pathlib import Path
import re, subprocess, sys

tracked=[Path(x) for x in subprocess.check_output(["git","ls-files"], text=True).splitlines()]
forbidden_exact={"AGENTS.md","docs/CURRENT-STATE.md","docs/DECISIONS.md","docs/ROADMAP.md","docs/TRACEABILITY.md","docs/ALPHA-IMPLEMENTATION.md","docs/ALPHA-USER-QA-GUIDE.md","docs/SPRINT-02-IMPLEMENTATION.md","docs/SPRINT-03-IMPLEMENTATION.md","docs/SPRINT-03-PLAN.md"}
forbidden_prefixes=("docs/internal/","docs/research/")
findings=[]
patterns=[
 ("private-key",re.compile(r"BEGIN [A-Z ]*PRIVATE KEY")),
 ("github-token",re.compile(r"(?:github_pat_|gh[pousr]_)[A-Za-z0-9_]")),
 ("aws-key",re.compile(r"AKIA[0-9A-Z]{16}")),
 ("windows-user-path",re.compile(r"C:\\\\Users\\\\",re.I)),
 ("linux-home-path",re.compile(r"/home/[A-Za-z0-9._-]+/")),
 ("wsl-path",re.compile(r"/mnt/[a-z]/",re.I)),
]
text_ext={".md",".txt",".yml",".yaml",".json",".toml",".rs",".js",".css",".html",".ps1",".sh",".env"}
for p in tracked:
    s=p.as_posix()
    if s in forbidden_exact or s.startswith(forbidden_prefixes): findings.append("forbidden public file: "+s)
    if p.suffix.lower() not in text_ext and p.name not in {".gitignore"}: continue
    data=p.read_text(encoding="utf-8",errors="ignore")
    for label,rx in patterns:
        if rx.search(data): findings.append(f"{label}: {s}")
if findings:
    print("PUBLIC SURFACE FAIL"); print("\n".join(sorted(set(findings)))); sys.exit(1)
print(f"PUBLIC SURFACE PASS ({len(tracked)} tracked files checked)")
