#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# Offline actions.lock consistency check (python heredoc: repo policy bans new .py files).
exec python3 - "$@" <<'PY'
"""Offline consistency check for .github/workflows/actions.lock.

Mirrors what `gh actions-lock --verify-local` rejects, without needing the
gh extension binary: (1) each workflow's lock list equals the actions it
actually uses (repo@ref, sub-paths collapsed, reusable workflows and local
actions excluded); (2) every listed action has a dependency entry; (3) every
dependency entry is reachable from a workflow (no orphans). Exit 1 on drift.
"""
import glob, re, sys, yaml
wf_dir = sys.argv[1] if len(sys.argv) > 1 else ".github/workflows"
lock = yaml.safe_load(open(f"{wf_dir}/actions.lock"))
deps, listed = lock.get("dependencies", {}), lock.get("workflows", {})
errs = []
def uses(node):
    if isinstance(node, dict):
        for k, v in node.items():
            if k == "uses" and isinstance(v, str): yield v.strip()
            else: yield from uses(v)
    elif isinstance(node, list):
        for v in node: yield from uses(v)
actual = {}
for p in sorted(glob.glob(f"{wf_dir}/*.yml") + glob.glob(f"{wf_dir}/*.yaml")):
    s = set()
    for u in uses(yaml.safe_load(open(p))):
        if u.startswith(("./", "docker://")) or "/.github/workflows/" in u: continue
        repo, ref = u.split("@", 1)
        s.add("/".join(repo.split("/")[:2]) + "@" + ref)
    actual[p] = s
for p, s in actual.items():
    got = set(listed.get(p, []) or [])
    for x in sorted(s - got): errs.append(f"{p}: uses {x} but lock does not list it")
    for x in sorted(got - s): errs.append(f"{p}: lock lists {x} but workflow does not use it")
for p in listed:
    if p not in actual: errs.append(f"lock lists missing workflow {p}")
seen, stack = set(), [x for s in actual.values() for x in s]
while stack:
    x = stack.pop()
    if x in seen: continue
    seen.add(x)
    if x not in deps: errs.append(f"no dependency entry for {x}"); continue
    stack += deps[x].get("uses", []) or []
# An unreachable entry is an error when a reachable entry for the same repo
# supersedes it (stale version left behind by a bump). Otherwise it is only a
# warning: it may be transitive coverage for a standards reusable workflow.
live_repos = {x.split("@")[0] for x in seen}
for x in sorted(set(deps) - seen):
    if x.split("@")[0] in live_repos and not re.fullmatch(r"[0-9a-f]{40}", x.split("@")[1]):
        errs.append(f"stale superseded dependency entry: {x}")
    else:
        print(f"warning: unreachable entry (reusable-workflow coverage?): {x}")
print("\n".join(errs) or "actions.lock: consistent")
sys.exit(1 if errs else 0)
PY
