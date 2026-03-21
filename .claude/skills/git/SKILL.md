# Git Skill

## When to use this skill
Use this skill when asked to:
- Summarize the difference between a feature branch and the default branch
- Write a commit message for a branch or set of changes
- Describe what a branch does, what it changes, or what it's for
- Review a diff or changelog in plain language

---

## Core Workflow

### 1. Identify the default branch
```bash
git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@' \
  || git remote show origin 2>/dev/null | grep 'HEAD branch' | awk '{print $NF}' \
  || echo "main"
```
Fall back to `main`, then `master` if the above fails.

### 2. Identify the current feature branch
```bash
git rev-parse --abbrev-ref HEAD
```

### 3. Get the merge base (branch point)
```bash
git merge-base HEAD <default_branch>
```
Always diff from the **merge base**, not the tip of the default branch. This isolates only the commits introduced by the feature branch, excluding any upstream changes merged in.

### 4. Collect the diff and log
```bash
# Stat summary (files changed, insertions, deletions)
git diff --stat $(git merge-base HEAD <default_branch>)

# Full diff (use sparingly for large repos — prefer --stat first)
git diff $(git merge-base HEAD <default_branch>)

# Commit log on this branch only
git log --oneline $(git merge-base HEAD <default_branch>)..HEAD
```

---

## Summarizing the Branch

Produce a short, plain-English summary covering:

1. **What changed** — files, modules, or systems affected
2. **Why it changed** — the purpose or problem being solved (infer from code and commit messages if not stated)
3. **Scope** — number of commits, files touched, rough size of the change

Keep the summary to 3–6 sentences. Avoid restating the diff line-by-line. Focus on intent and impact, not mechanics.

### Example summary format
> This branch adds rate-limit handling to the API client. It introduces a retry loop with exponential backoff in `client.py` and updates `config.py` to expose a `MAX_RETRIES` setting. Three commits touch two files. No tests were changed.

---

## Writing a Commit Message

Use the **imperative mood**, present tense, and keep it direct. Do not over-explain.

### Format
```
<Short subject line — 50 chars or fewer>

<Optional body — wrap at 72 chars. Explain WHAT and WHY, not HOW.
   Omit if the subject line is self-sufficient.>
```

### Rules
- **Subject line**: imperative mood ("Add", "Fix", "Remove", "Update") — not "Added" or "Adding"
- **No period** at the end of the subject line
- **Body** (optional): explain *why* the change was made if it isn't obvious; omit implementation details that are visible in the diff
- **No fluff**: avoid "This commit...", "As per discussion...", "Various improvements"
- If the branch has multiple unrelated concerns, note them with bullet points in the body

### Example commit messages

Single-concern branch:
```
Add exponential backoff to API client retries
```

Multi-concern or non-obvious branch:
```
Refactor auth middleware to support token refresh

The previous implementation rejected expired tokens immediately.
This change adds a refresh attempt before returning 401, matching
the behavior expected by the mobile clients.
```

---

## Quick Reference Commands

| Goal | Command |
|---|---|
| Default branch | `git symbolic-ref refs/remotes/origin/HEAD \| sed 's@.*/@@'` |
| Current branch | `git rev-parse --abbrev-ref HEAD` |
| Merge base | `git merge-base HEAD <default>` |
| Files changed | `git diff --name-only $(git merge-base HEAD <default>)` |
| Stat summary | `git diff --stat $(git merge-base HEAD <default>)` |
| Commit log | `git log --oneline $(git merge-base HEAD <default>)..HEAD` |
| Full diff | `git diff $(git merge-base HEAD <default>)` |

---

## Notes & Edge Cases

- **Detached HEAD**: if `git rev-parse --abbrev-ref HEAD` returns `HEAD`, ask the user which branch they're working on.
- **Large diffs**: for repos with thousands of changed lines, use `--stat` and `--name-only` first. Only fetch the full diff for files relevant to the summary.
- **Merge commits on the branch**: use `git log --no-merges` to exclude them from the commit list when writing the message.
- **Monorepos**: scope the diff to the relevant subdirectory with `git diff <merge-base> -- <path/to/dir>` if the branch touches only one package.
- **No remote**: if there's no remote, fall back to comparing against `main` or `master` locally.