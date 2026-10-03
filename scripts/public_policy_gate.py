#!/usr/bin/env python3
from pathlib import Path
import sys
required=["README.md","LICENSE","NOTICE","SECURITY.md","CONTRIBUTING.md","Cargo.toml","Cargo.lock",".github/pull_request_template.md"]
missing=[x for x in required if not Path(x).is_file()]
if missing:
    print("POLICY FAIL: missing required public files: "+", ".join(missing)); sys.exit(1)
cargo=Path("Cargo.toml").read_text(encoding="utf-8")
errors=[]
if 'unsafe_code = "forbid"' not in cargo: errors.append("workspace must keep unsafe_code=forbid")
if 'rust-version = "1.98"' not in cargo: errors.append("approved Rust baseline missing")
if errors:
    print("POLICY FAIL"); print("\n".join(errors)); sys.exit(1)
print("POLICY PASS")
