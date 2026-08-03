# Release notifications and publishing

This file contains the release-related notifications and instructions referenced from the repository README.

## What the CI does

The GitHub Actions CI builds platform installers for Windows, macOS (Apple Silicon + Intel), and Linux and attaches the generated installers to a draft GitHub Release.

## Triggering the publish workflow

You can trigger the workflow in one of these ways:

- From the GitHub web UI: go to Actions → find the workflow named `publish` (or the publish workflow used by this repo) → Run workflow → select branch `release` and start the run.
- Push a commit to the `release` branch; the workflow should run on pushes to that branch if configured.
- Use the workflow dispatch URL / run page directly. Example run (most recent run link provided by the user):

  https://github.com/Mingli29M/Northstar-Minecraft-Launcher/actions/runs/30821836315

## Required pre-checks

1. Ensure the repository has Settings → Actions → Workflow permissions → Read and write (Actions needs write permissions to create/update draft releases and attach artifacts).
2. Ensure any secrets required by the publish workflow (signing keys, tokens, etc.) are set in Settings → Secrets.
3. Ensure the branch `release` exists in the repo (create/push it if it doesn't). The repo now has a `release` branch created.

## What happens after CI completes

- The workflow creates a draft GitHub Release and attaches built installers as artifacts. You can find it under Releases → Drafts in the repository.
- Download the artifacts from the draft release — you do not need a local macOS machine to build macOS artifacts.

## Troubleshooting

- If the workflow fails or artifacts are missing, open the Actions run and inspect the job logs for the failing job.
- If Actions cannot write the draft release, double-check the Workflow permissions setting mentioned above and any permissions required by actions used in the workflow (e.g., a GitHub token with write permissions).

## Want me to

- Trigger the `publish` workflow for you now.
- Inspect the Actions run logs at the provided URL and report back on failures or the draft release state.
- Open a pull request that adds or refines these release instructions.
