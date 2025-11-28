# GitHub Copilot Instructions for FMMLoader26

## Project Overview

FMMLoader26 is a Football Manager 2026 mod loader built with:
- **Frontend**: React + TypeScript + Vite + Tailwind CSS
- **Backend**: Rust + Tauri v2
- **UI Components**: shadcn/ui

## Git Workflow & Branch Strategy

### Branch Naming Convention

Always work on feature branches, never commit directly to `main`. Use these prefixes:

- `feat/` - New features (e.g., `feat/mods-folder-open-btn`)
- `fix/` - Bug fixes (e.g., `fix/toast-notification-spam`)
- `docs/` - Documentation updates (e.g., `docs/update-readme`)
- `refactor/` - Code refactoring (e.g., `refactor/mod-manager-cleanup`)
- `test/` - Adding or updating tests (e.g., `test/import-validation`)
- `chore/` - Maintenance tasks (e.g., `chore/update-dependencies`)

### Standard Workflow

1. **Create a feature branch** from main:
   ```bash
   git checkout main
   git pull origin main
   git checkout -b feat/your-feature-name
   ```

2. **Make changes** on the feature branch

3. **Commit with descriptive messages**:
   ```bash
   git add <files>
   git commit -m "feat: add feature description
   
   - Detail 1
   - Detail 2
   - Detail 3"
   ```

4. **Merge to main** when ready:
   ```bash
   git checkout main
   git merge feat/your-feature-name --no-ff -m "Merge feat/your-feature-name into main"
   ```

5. **Optional**: Delete the feature branch after merging:
   ```bash
   git branch -d feat/your-feature-name
   ```

### Commit Message Format

Follow conventional commits format:

```
<type>: <short description>

<optional detailed description>
- Bullet point 1
- Bullet point 2
```

**Types:**
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation only
- `style:` - Formatting, missing semicolons, etc.
- `refactor:` - Code restructuring
- `test:` - Adding tests
- `chore:` - Maintenance

**Examples:**
```
feat: add Open Mods Folder button to settings

- Add open_mods_folder Tauri command
- Add openModsFolder function to TypeScript hooks
- Add UI button in Settings panel
```

```
fix: prevent multiple toast notifications on import

- Add toast ID to prevent duplicates
- Update import handler to use single toast
```

## Code Style & Guidelines

### TypeScript/React

- Use functional components with hooks
- Use TypeScript for type safety
- Follow existing shadcn/ui patterns for UI components
- Keep components modular and reusable
- Use async/await for Tauri commands
- Handle errors gracefully with try/catch and user-friendly messages

### Rust

- Follow Rust best practices and clippy suggestions
- Use proper error handling with `Result<T, String>`
- Add tracing logs for debugging (use `tracing::info!`, `tracing::error!`, etc.)
- Document public functions
- Keep modules focused and organized

### File Organization

```
src/
  components/     - React UI components
  hooks/          - Custom React hooks (including Tauri commands)
  lib/            - Utility functions
  
src-tauri/src/
  main.rs         - Tauri commands and app initialization
  config.rs       - Configuration management
  mod_manager.rs  - Mod installation and management
  import.rs       - Mod import logic
  conflicts.rs    - Conflict detection
  restore.rs      - Backup/restore functionality
  name_fix.rs     - FM Name Fix utility
  types.rs        - Shared type definitions
```

## Making Changes

### Adding a New Tauri Command

1. **Create branch**: `git checkout -b feat/new-command`

2. **Add Rust command** in `src-tauri/src/main.rs`:
   ```rust
   #[tauri::command]
   fn your_command_name(param: String) -> Result<ReturnType, String> {
       // Implementation
       Ok(result)
   }
   ```

3. **Register command** in `main()`:
   ```rust
   .invoke_handler(tauri::generate_handler![
       // ... existing commands
       your_command_name,
   ])
   ```

4. **Add TypeScript binding** in `src/hooks/useTauri.ts`:
   ```typescript
   yourCommandName: (param: string) => 
     safeInvoke<ReturnType>("your_command_name", { param }),
   ```

5. **Use in React** component:
   ```typescript
   const result = await tauriCommands.yourCommandName(value);
   ```

6. **Commit and merge**:
   ```bash
   git add .
   git commit -m "feat: add your command description"
   git checkout main
   git merge feat/new-command --no-ff
   ```

### Adding UI Components

1. Use existing shadcn/ui components from `src/components/ui/`
2. Follow the pattern in `App.tsx` for layout and structure
3. Use Tailwind CSS for styling
4. Maintain dark mode compatibility
5. Add proper error handling and loading states
6. Use tooltips for better UX where appropriate

## Testing & Building

### Development
```bash
npm run tauri dev
```

### Build
```bash
npm run tauri build
```

### Linting (if configured)
```bash
npm run lint
cargo clippy
```

## Important Notes

- **Never commit directly to main** - always use feature branches
- **Test changes** before committing (run in dev mode)
- **Keep commits focused** - one feature/fix per commit when possible
- **Update documentation** if adding user-facing features
- **Handle errors gracefully** - show user-friendly messages
- **Log important operations** - use tracing in Rust, console in TypeScript
- **Maintain cross-platform compatibility** - test on Windows, macOS, Linux where possible

## Project-Specific Patterns

### Mods Storage Location
Mods are stored in: `{AppData}/FMMLoader26/mods/`
- Windows: `%APPDATA%\FMMLoader26\mods\`
- macOS: `~/Library/Application Support/FMMLoader26/mods/`
- Linux: `~/.local/share/FMMLoader26/mods/`

### Configuration
Config is stored as JSON in: `{AppData}/FMMLoader26/config.json`

### Logging
Logs are stored in: `{AppData}/FMMLoader26/logs/`
- Last 10 sessions are kept automatically

### Restore Points
Backups are stored in: `{AppData}/FMMLoader26/restore_points/`
- Last 10 restore points are kept automatically

## Getting Help

- Review existing code patterns in the repository
- Check `README.md` for user-facing documentation
- Check `BUILD.md` for build instructions
- Review recent commits for examples of good practices


<!-- CLAVIX:START -->
# Clavix Instructions for GitHub Copilot

These instructions enhance GitHub Copilot's understanding of Clavix prompt optimization workflows available in this project.

---

## ⛔ CLAVIX MODE ENFORCEMENT

**CRITICAL: Know which mode you're in and STOP at the right point.**

**OPTIMIZATION workflows** (NO CODE ALLOWED):
- Fast/deep optimization - Prompt improvement only
- Your role: Analyze, optimize, show improved prompt, **STOP**
- ❌ DO NOT implement the prompt's requirements
- ✅ After showing optimized prompt, tell user: "Run `/clavix:implement --latest` to implement"

**PLANNING workflows** (NO CODE ALLOWED):
- Conversational mode, requirement extraction, PRD generation
- Your role: Ask questions, create PRDs/prompts, extract requirements
- ❌ DO NOT implement features during these workflows

**IMPLEMENTATION workflows** (CODE ALLOWED):
- Only after user runs execute/implement commands
- Your role: Write code, execute tasks, implement features
- ✅ DO implement code during these workflows

**If unsure, ASK:** "Should I implement this now, or continue with planning?"

See `.clavix/instructions/core/clavix-mode.md` for complete mode documentation.

---

## 📁 Detailed Workflow Instructions

For complete step-by-step workflows, see `.clavix/instructions/`:

| Workflow | Instruction File | Purpose |
|----------|-----------------|---------|
| **Conversational Mode** | `workflows/start.md` | Natural requirements gathering through discussion |
| **Extract Requirements** | `workflows/summarize.md` | Analyze conversation → mini-PRD + optimized prompts |
| **Prompt Optimization** | `workflows/improve.md` | Intent detection + quality assessment + auto-depth selection |
| **PRD Generation** | `workflows/prd.md` | Socratic questions → full PRD + quick PRD |
| **Mode Boundaries** | `core/clavix-mode.md` | Planning vs implementation distinction |

**Troubleshooting:**
- `troubleshooting/jumped-to-implementation.md` - If you started coding during planning
- `troubleshooting/skipped-file-creation.md` - If files weren't created
- `troubleshooting/mode-confusion.md` - When unclear about planning vs implementation

**When detected:** Reference the corresponding `.clavix/instructions/workflows/{workflow}.md` file.

**⚠️ GitHub Copilot Limitation:** If Write tool unavailable, provide file content with clear "save to" instructions for user.

---

## 📋 Clavix Commands (v5)

### Setup Commands (CLI)
| Command | Purpose |
|---------|---------|
| `clavix init` | Initialize Clavix in a project |
| `clavix update` | Update templates after package update |
| `clavix diagnose` | Check installation health |
| `clavix version` | Show version |

### Workflow Commands (Slash Commands)
All workflows are executed via slash commands:

| Slash Command | Purpose |
|---------------|---------|
| `/clavix:improve` | Optimize prompts (auto-selects depth) |
| `/clavix:prd` | Generate PRD through guided questions |
| `/clavix:plan` | Create task breakdown from PRD |
| `/clavix:implement` | Execute tasks or prompts (auto-detects source) |
| `/clavix:start` | Begin conversational session |
| `/clavix:summarize` | Extract requirements from conversation |

### Agentic Utilities (Project Management)
| Utility | Purpose |
|---------|---------|
| `/clavix:verify` | Check implementation against PRD requirements |
| `/clavix:archive` | Archive completed work to `.clavix/archive/` |

---

## 🔄 Prompt Lifecycle Workflow

**Prompt Lifecycle:**
1. **Optimize**: `/clavix:improve` → Analyzes and improves your prompt
2. **Review**: Agent reads `.clavix/outputs/prompts/*.md` to list saved prompts
3. **Execute**: `/clavix:implement --latest` → Implement when ready
4. **Cleanup**: Agent deletes old prompt files from `.clavix/outputs/prompts/`

---

## 🔄 Standard Workflow

**Clavix follows this progression:**

```
PRD Creation → Task Planning → Implementation → Archive
```

**Detailed steps:**

1. **Planning Phase** - `/clavix:prd` or `/clavix:start` → `/clavix:summarize`
2. **Task Preparation** - `/clavix:plan` transforms PRD into tasks.md
3. **Implementation Phase** - `/clavix:implement` executes tasks systematically
4. **Completion** - `/clavix:archive` archives completed work

**Key principle:** Planning workflows create documents. Implementation workflows write code.

---

## 🎯 Quality Dimensions

When analyzing prompts, consider these 5 dimensions:

- **Clarity**: Is the objective clear and unambiguous?
- **Efficiency**: Concise without losing critical information?
- **Structure**: Information organized logically (context → requirements → constraints → output)?
- **Completeness**: All specs provided (persona, format, tone, success criteria)?
- **Actionability**: Can AI take immediate action?

**Reference:** See `workflows/improve.md` for complete quality assessment patterns

---

## 💡 Best Practices for GitHub Copilot

1. **Suggest appropriate workflow** - `/clavix:improve` for prompts, `/clavix:prd` for strategic planning
2. **Reference instruction files** - Point to `.clavix/instructions/workflows/*.md` instead of recreating steps
3. **Respect mode boundaries** - Planning mode = no code, Implementation mode = write code
4. **Use quality dimensions** - Apply 5-dimension assessment principles in responses
5. **Guide users to slash commands** - Recommend appropriate `/clavix:*` commands for their needs

---

## ⚠️ Common Mistakes

### ❌ Jumping to implementation during planning
**Wrong:** User discusses feature → Copilot generates code immediately

**Right:** User discusses feature → Suggest `/clavix:prd` or `/clavix:start` → Create planning docs first

### ❌ Not suggesting Clavix workflows
**Wrong:** User asks "How should I phrase this?" → Copilot provides generic advice

**Right:** User asks "How should I phrase this?" → Suggest `/clavix:improve` for quality assessment

### ❌ Recreating workflow steps inline
**Wrong:** Copilot explains entire PRD generation process in chat

**Right:** Copilot references `.clavix/instructions/workflows/prd.md` and suggests running `/clavix:prd`

---

## 🔗 Integration with GitHub Copilot

When users ask for help with prompts or requirements:

1. **Detect need** - Identify if user needs planning, optimization, or implementation
2. **Suggest slash command** - Recommend appropriate `/clavix:*` command
3. **Explain benefit** - Describe expected output and value
4. **Help interpret** - Assist with understanding Clavix-generated outputs
5. **Apply principles** - Use quality dimensions in your responses

**Example flow:**
```
User: "I want to build a dashboard but I'm not sure how to phrase the requirements"
Copilot: "I suggest running `/clavix:start` to begin conversational requirements gathering.
This will help us explore your needs naturally, then we can extract structured requirements
with `/clavix:summarize`. Alternatively, if you have a rough idea, try:
`/clavix:improve 'Build a dashboard for...'` for quick optimization."
```

---

**Artifacts stored under `.clavix/`:**
- `.clavix/outputs/<project>/` - PRDs, tasks, prompts
- `.clavix/config.json` - Project configuration

---

**For complete workflows:** Always reference `.clavix/instructions/workflows/{workflow}.md`

**For troubleshooting:** Check `.clavix/instructions/troubleshooting/`

**For mode clarification:** See `.clavix/instructions/core/clavix-mode.md`

<!-- CLAVIX:END -->
