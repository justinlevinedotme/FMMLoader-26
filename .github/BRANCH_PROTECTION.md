# Branch Protection Settings

This document describes the recommended branch protection settings for the `main` branch to ensure code quality and prevent breaking changes.

## Accessing Branch Protection Settings

1. Go to the repository on GitHub
2. Navigate to **Settings** → **Branches**
3. Under "Branch protection rules", click **Add rule** (or edit existing rule for `main`)

## Recommended Settings for `main` Branch

### Branch name pattern
```
main
```

### Protect matching branches

#### ✅ Require a pull request before merging
- **Required approvals**: 1
- ☑️ Dismiss stale pull request approvals when new commits are pushed
- ☐ Require review from Code Owners (optional - only if you have a CODEOWNERS file)
- ☐ Restrict who can dismiss pull request reviews (optional)
- ☐ Allow specified actors to bypass required pull requests (not recommended)
- ☐ Require approval of the most recent reviewable push

#### ✅ Require status checks to pass before merging
- ☑️ Require branches to be up to date before merging

**Required status checks** (add these after the CI workflow runs for the first time):
- `test` (from the CI workflow)

> **Note**: Status checks will only appear in the list after they have run at least once. After your first PR with the CI workflow, you can add the `test` check as required.

#### ✅ Require conversation resolution before merging
This ensures all review comments are addressed before merging.

#### ✅ Require signed commits (optional but recommended)
Ensures all commits are cryptographically signed.

#### ✅ Require linear history
Prevents merge commits and enforces a clean, linear git history.

#### ☐ Require deployments to succeed before merging
Not needed for this project.

#### ☐ Lock branch
Not recommended - this would prevent all pushes.

#### ☐ Do not allow bypassing the above settings
Recommended: Leave unchecked so repository admins can bypass in emergencies.

#### ✅ Restrict who can push to matching branches
- Add specific users/teams who can push directly (typically only maintainers)
- This prevents accidental direct pushes to main

#### ✅ Allow force pushes
- ☐ Everyone (not recommended)
- ☐ Specify who can force push (optional - only for maintainers in emergencies)
- Recommended: Leave unchecked to prevent force pushes entirely

#### ✅ Allow deletions
- ☐ Do not allow deletion of the main branch
Recommended: Leave unchecked to prevent accidental deletion.

## Summary

The most important settings for this project are:

1. **Require pull requests** with at least 1 approval
2. **Require status checks** (CI must pass)
3. **Require branches to be up to date** before merging
4. **Require linear history** (prevents merge commits)
5. **Require conversation resolution** (all comments must be addressed)

These settings ensure:
- All code is reviewed before merging
- CI (build, lint, tests) passes on all PRs
- The git history remains clean and linear
- Breaking changes are caught before reaching production

## Initial Setup

Since status checks are not visible until they run once:

1. First, merge the CI workflow changes
2. Create a test PR to trigger the CI workflow
3. Once the `test` job appears in the status checks list, add it as a required check
4. Apply the remaining branch protection settings

## Testing Branch Protection

After setting up branch protection, test it by:

1. Creating a new branch with a small change
2. Opening a PR
3. Verify that:
   - CI runs automatically
   - You cannot merge without CI passing
   - You cannot merge without approval (if you have multiple maintainers)
   - Direct pushes to `main` are blocked

---

**Last Updated**: 2025-11-17
