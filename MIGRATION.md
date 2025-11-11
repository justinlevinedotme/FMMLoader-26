# Migration Guide

## Upgrading from v1.0.0 to v1.1.0+

### Important: One-Time Manual Update Required

**Version 1.0.0 does not include the auto-updater**, so you'll need to manually update to v1.1.0. After this one-time manual update, all future updates will be automatic!

### Why Manual Update is Needed

The auto-updater feature was added in v1.1.0. Since v1.0.0 doesn't have this feature built-in, it can't detect or install updates automatically. This is a one-time issue - once you're on v1.1.0 or later, you'll get automatic update notifications.

### How to Update from v1.0.0

1. **Download v1.1.0** from the [Releases page](https://github.com/justinlevinedotme/FMMLoader-26/releases)
   - **Windows**: Download `FMMLoader26_1.1.0_x64-setup.exe`
   - **macOS**: Download `FMMLoader26_1.1.0_aarch64.dmg` (Apple Silicon) or `FMMLoader26_1.1.0_x64.dmg` (Intel)
   - **Linux**: Download `fmmloader26_1.1.0_amd64.AppImage`

2. **Close the current app** if it's running

3. **Install the new version**:
   - **Windows**: Run the installer - it will automatically replace the old version
   - **macOS**: Drag the new app to Applications, replacing the old one
   - **Linux**: Replace the old AppImage with the new one

4. **Launch v1.1.0** - Your settings and mods will be preserved!

### What's Preserved During Update

✅ All your mods and their enabled/disabled state
✅ Game directory and user directory settings
✅ Dark mode preference
✅ FM Name Fix installation (if installed)
✅ All backup and restore points

Everything is stored in the app data directory, which is separate from the app itself:
- **Windows**: `%APPDATA%\FMMLoader26`
- **macOS**: `~/Library/Application Support/FMMLoader26`
- **Linux**: `~/.local/share/FMMLoader26`

### After Updating to v1.1.0

Once you're on v1.1.0 or later, the app will:
- ✨ Automatically check for updates when you start the app
- ✨ Show a native dialog when an update is available
- ✨ Let you install updates with one click
- ✨ No more manual downloads needed!

### What's New in v1.1.0

**New Features:**
- 🔄 **Auto-updater** - Automatic update notifications and one-click updates
- 🌙 **Persistent Dark Mode** - Your dark mode preference now saves between sessions
- 🛠️ **FM Name Fix Utility** - One-click installer for FM Name Fix with automatic backup
- 📦 **Local Build Scripts** - Easy debug builds for testing without releases

**Improvements:**
- Better error handling throughout the app
- Improved logging for troubleshooting
- More detailed documentation

### Troubleshooting

**Q: Will I lose my mods?**
A: No! All mod data is stored separately from the app. Your mods will remain intact.

**Q: Do I need to uninstall v1.0.0 first?**
A: No, the installer/app replacement will handle it automatically.

**Q: What if I have issues after updating?**
A: Your app data (including backups) is preserved. You can:
1. Check the logs in Settings → "Open Logs Folder"
2. Create a restore point before updating if you want extra safety
3. Report issues on [GitHub Issues](https://github.com/justinlevinedotme/FMMLoader-26/issues)

**Q: Can I go back to v1.0.0?**
A: Yes, you can download v1.0.0 again from the releases page. Your settings will work with both versions.

### Future Updates (v1.1.0+)

From v1.1.0 onwards, you'll see a dialog like this when updates are available:

```
Update Available
FMMLoader26 v1.2.0 is now available.
You are currently running v1.1.0.

Would you like to install this update?
[Install] [Skip]
```

Click **Install** and the app will:
1. Download the update in the background
2. Install it automatically
3. Ask you to restart the app
4. Done! You're on the latest version

---

**Still on v1.0.0?**
[Download v1.1.0 now](https://github.com/justinlevinedotme/FMMLoader-26/releases/tag/v1.1.0) to get automatic updates!
