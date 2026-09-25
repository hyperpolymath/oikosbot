#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# Re-run every `run:` step of .github/workflows/workflow-linter.yml locally,
# verbatim, so a fix can be checked before merge (only merges trigger Actions).
# Needs python3 + PyYAML. Exit non-zero if any step fails.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"
python3 - <<'PY' > /tmp/linter-steps.tsv
import yaml, json
wf = yaml.safe_load(open(".github/workflows/workflow-linter.yml"))
for job in wf["jobs"].values():
    for s in job["steps"]:
        if "run" in s: print(json.dumps([s.get("name", "?"), s["run"]]))
PY
fail=0
while IFS= read -r line; do
  name=$(python3 -c 'import json,sys;print(json.loads(sys.argv[1])[0])' "$line")
  body=$(python3 -c 'import json,sys;print(json.loads(sys.argv[1])[1])' "$line")
  if out=$(bash -e -o pipefail -c "$body" 2>&1); then echo "PASS  $name"
  else echo "FAIL  $name"; echo "$out" | sed 's/^/      /' | tail -20; fail=1; fi
done < /tmp/linter-steps.tsv
bash tools/ci/lockcheck.sh >/dev/null && echo "PASS  lockcheck.sh (actions.lock consistency)" || { echo "FAIL  lockcheck.sh"; fail=1; }
exit $fail
