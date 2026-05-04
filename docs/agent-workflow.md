# Codex Issue Workflow

This repository has a local issue orchestrator at `scripts/agent-run`. It prepares one isolated Git worktree per GitHub issue and generates fixed prompts for the test, dev, and review agents.

## Commands

Prepare one issue:

```bash
scripts/agent-run issue 123
```

Prepare and execute the full TDD loop:

```bash
scripts/agent-run issue 123 --execute
```

Resume an existing issue run:

```bash
scripts/agent-run resume 123
```

Open a draft PR after the run reaches `ready_for_pr`:

```bash
scripts/agent-run pr 123
```

Show state:

```bash
scripts/agent-run status
scripts/agent-run status 123
```

Prepare multiple issues:

```bash
scripts/agent-run issues 123 124 125
```

Prepare multiple issues and execute up to three issue workflows concurrently:

```bash
scripts/agent-run issues 123 124 125 --execute --parallel 3
```

## Workflow

Each issue run uses this pipeline:

1. Fetch issue details with `gh issue view`.
2. Create or reuse a branch named `codex/issue-<number>-<title>`.
3. Create a worktree under `../publisher-agent-worktrees`.
4. Write issue context to `.agent-context/issue.md` inside the worktree.
5. Run the test agent, which may only write tests and must finish with `TEST_STATUS: red` or `TEST_STATUS: ci-gated`.
6. Run the dev agent, which implements the feature and must finish with `DEV_STATUS: green`.
7. Run configured checks from `agent-run.json`.
8. Run the review agent, which must finish with `REVIEW_STATUS: pass` or write findings to `.agent-context/review-notes.md`.
9. Loop dev/review up to `max_review_loops`.
10. Mark the issue `ready_for_pr` when checks and review pass.

## State And Logs

Runtime state is ignored by Git and stored under:

```text
.agent-run/issues/<number>/
```

Important files:

```text
state.json
issue.md
prompts/test.md
prompts/dev.md
prompts/review.md
logs/*.log
logs/*-final.md
```

## Configuration

Edit `agent-run.json` to change:

- `base_branch`
- `branch_prefix`
- `worktree_root`
- `max_review_loops`
- `ci_gated_labels`
- check commands
- Codex command/model/sandbox settings
- draft PR behavior and labels

The default checks match the current CI-critical local checks where practical:

```bash
cargo test --workspace
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
npm --prefix apps/web run check
```

CI-gated issues are useful for release, platform, or GitHub Actions failures that cannot be reproduced reliably on a local developer machine. For those issues the test phase may produce workflow or artifact-validation checks that pass locally, then the PR CI run becomes the required proof.
