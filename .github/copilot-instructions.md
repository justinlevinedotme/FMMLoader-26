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
