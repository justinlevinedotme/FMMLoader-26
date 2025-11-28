# Crowdin Setup Guide

This guide will help you set up Crowdin for community-driven translations of FMMLoader26.

## What You'll Get

- **Automatic sync** of source files (English) to Crowdin when you push to `main`
- **Daily imports** of completed translations via pull requests
- **Community contributors** can translate through Crowdin's web interface
- **Translation memory** to maintain consistency across updates

## Prerequisites

1. A Crowdin account (free for open source projects)
2. GitHub repository admin access
3. 10 minutes of setup time

---

## Step 1: Create a Crowdin Project

1. Go to https://crowdin.com/ and sign in (or create an account)
2. Click **Create Project**
3. Fill in project details:
   - **Project name**: FMMLoader26
   - **Source language**: English
   - **Target languages**: Add these languages:
     - German (de)
     - Korean (ko)
     - Turkish (tr)
     - Portuguese, Portugal (pt-PT)
   - **Project type**: Choose "Software Localization"
   - **Public project**: Enable this for open source

4. After creating, note your **Project ID** from the project settings
   - Go to **Settings** → **API** tab
   - Copy the **Project ID** number

## Step 2: Generate Crowdin API Token

1. In Crowdin, click your avatar → **Account Settings**
2. Go to **API** tab
3. Click **New Token**
4. Name it: `FMMLoader26 GitHub Actions`
5. Scopes needed:
   - ✅ Projects: Read, Write
   - ✅ Source files & strings: Read, Write
   - ✅ Translations: Read, Write
6. Click **Create** and **copy the token** (you won't see it again!)

## Step 3: Add Secrets to GitHub

1. Go to your GitHub repository
2. Navigate to **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret** and add these two secrets:

   **Secret 1:**
   - Name: `CROWDIN_PROJECT_ID`
   - Value: Your project ID from Step 1

   **Secret 2:**
   - Name: `CROWDIN_PERSONAL_TOKEN`
   - Value: Your API token from Step 2

## Step 4: Initialize Crowdin Files

Run this command locally to upload your source files for the first time:

```bash
# Install Crowdin CLI
npm install -g @crowdin/cli

# Upload source files to Crowdin
crowdin upload sources --token YOUR_TOKEN_HERE --project-id YOUR_PROJECT_ID_HERE
```

Or use the GitHub Actions workflow:
1. Go to **Actions** tab in GitHub
2. Select **Crowdin Sync** workflow
3. Click **Run workflow**
4. Choose **upload** mode
5. Click **Run workflow**

## Step 5: Configure Target Languages in Crowdin

1. In your Crowdin project, go to **Settings** → **Languages**
2. Ensure these languages are added:
   - 🇩🇪 German (de)
   - 🇰🇷 Korean (ko)
   - 🇹🇷 Turkish (tr)
   - 🇵🇹 Portuguese, Portugal (pt-PT)

## Step 6: Invite Contributors

### Make Your Project Public (Recommended for Open Source)

1. Go to **Settings** → **General**
2. Under **Access**, select **Public project**
3. Enable **Join workflow** to let anyone start translating
4. Share your project URL: `https://crowdin.com/project/fmmloader26`

### Or Invite Specific Translators

1. Go to **Members** tab
2. Click **Invite**
3. Enter email addresses
4. Select role: **Translator** or **Proofreader**

---

## How It Works

### Automatic Workflow

1. **You update** `src/locales/en.json` and push to `main`
2. **GitHub Actions** automatically uploads the new English strings to Crowdin
3. **Translators** see the new strings in Crowdin and translate them
4. **Every day**, completed translations are downloaded and a PR is created
5. **You review** and merge the translation PR

### Manual Sync

You can also trigger syncs manually:

- **Actions** → **Crowdin Sync** → **Run workflow**
  - Choose `upload` to push source changes
  - Choose `download` to fetch translations immediately

---

## Translation Files

### What Gets Translated

Currently configured to translate:
- ✅ `src/locales/en.json` (Frontend translations)
- ✅ `locales/en.yml` (Backend translations)

---

## Best Practices

### For Maintainers

1. **Always edit English files only** - Never edit translated files directly
2. **Use context comments** - Add comments in JSON to help translators
3. **Review translation PRs** - Check for formatting issues before merging
4. **Keep strings atomic** - Break long sentences into smaller translatable units

### For Contributors

1. **Don't edit JSON/YAML files directly** - Use Crowdin instead
2. **Translation PRs are auto-generated** - Don't create manual translation PRs

---

## Adding New Languages

Want to support more languages?

1. **Add to Crowdin project**:
   - Settings → Languages → Add language

2. **Update `crowdin.yml`**:
   ```yaml
   languages_mapping:
     locale_with_underscore:
       pt-PT: pt-PT
       de: de
       ko: ko
       tr: tr
       es: es  # Add new language code
   ```

3. **Update your app's language selector** to include the new language

---

## Troubleshooting

### "Workflow failed to upload sources"

- Check that `CROWDIN_PROJECT_ID` and `CROWDIN_PERSONAL_TOKEN` secrets are set correctly
- Verify your token has the required scopes

### "Translation PR not created"

- Check that your Crowdin project has completed translations
- Verify the workflow has `pull-requests: write` permission
- Check Actions logs for errors

### "Translation files not syncing"

- Ensure `crowdin.yml` paths match your actual file structure
- Run `crowdin upload sources` locally to test configuration

---

## Testing the Setup

1. Make a small change to `src/locales/en.json`:
   ```json
   {
     "test": "Hello Crowdin!"
   }
   ```

2. Commit and push to `main`

3. Check Crowdin - you should see the new string

4. Translate it in Crowdin

5. Wait for the daily sync or manually trigger download

6. Review the auto-generated PR with translations

---

## Resources

- [Crowdin GitHub Integration Docs](https://support.crowdin.com/github-integration/)
- [Crowdin CLI Documentation](https://crowdin.github.io/crowdin-cli/)
- [Crowdin for Open Source](https://crowdin.com/page/open-source-project-setup-request)

---

## Questions?

- Crowdin docs: https://support.crowdin.com/
- GitHub Discussions: Create a discussion in this repo
- Issues: Open an issue for technical problems

Happy translating! 🌍
