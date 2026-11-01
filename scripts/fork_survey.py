"""Survey every fork of an upstream GitHub repository for commits not in upstream.

Why: the upstream (mixelpixx/Konnect) is archived and has ~200 forks. Most forks are
untouched copies. This lists, for every branch of every fork, how many commits it is
ahead of / behind upstream main, so only forks with real new work are examined by hand.

Usage: python3 -I fork_survey.py OWNER/REPO OUT.json
Requires an authenticated `gh` CLI. Uses the cross-fork compare API.
"""
import json
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor


def gh(path):
    """Single (non-paginated) API object, or None on error."""
    r = subprocess.run(["gh", "api", path], capture_output=True, text=True)
    if r.returncode != 0:
        return None
    return json.loads(r.stdout)


def gh_list(path):
    """All items of a paginated list endpoint."""
    r = subprocess.run(["gh", "api", "--paginate", "--jq", ".[]", path],
                       capture_output=True, text=True)
    if r.returncode != 0:
        return []
    return [json.loads(line) for line in r.stdout.splitlines() if line.strip()]


def survey_fork(upstream, base_branch, fork):
    name = fork["full_name"]
    owner = fork["owner"]["login"]
    repo = fork["name"]
    branches = gh_list(f"repos/{name}/branches?per_page=100")
    res = {"fork": name, "pushed_at": fork["pushed_at"], "stars": fork["stargazers_count"],
           "default_branch": fork["default_branch"], "branches": []}
    for b in branches:
        cmp = gh(f"repos/{upstream}/compare/{base_branch}...{owner}:{repo}:{b['name']}")
        if cmp is None:
            res["branches"].append({"name": b["name"], "error": True})
            continue
        res["branches"].append({
            "name": b["name"],
            "ahead": cmp["ahead_by"],
            "behind": cmp["behind_by"],
            "files": len(cmp.get("files", [])),
            "commits": [
                {"sha": c["sha"][:10], "date": c["commit"]["committer"]["date"],
                 "author": c["commit"]["author"]["name"],
                 "msg": c["commit"]["message"].split("\n")[0]}
                for c in cmp.get("commits", [])
            ],
        })
    return res


def main():
    upstream, out = sys.argv[1], sys.argv[2]
    base = gh(f"repos/{upstream}")["default_branch"]
    forks = gh_list(f"repos/{upstream}/forks?per_page=100")
    # include second-level forks (forks of forks)
    nested = []
    for f in forks:
        if f["forks_count"]:
            nested += gh_list(f"repos/{f['full_name']}/forks?per_page=100")
    forks += nested
    with ThreadPoolExecutor(8) as ex:
        results = list(ex.map(lambda f: survey_fork(upstream, base, f), forks))
    json.dump(results, open(out, "w"), indent=1)
    print(f"{len(forks)} forks surveyed")


if __name__ == "__main__":
    main()
