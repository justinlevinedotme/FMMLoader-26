# Platform-Specific Mod Support & Skins Placement

## Overview

This document describes the new features added to FMMLoader26 for handling platform-specific mods and correct placement of skins-type mods.

## Features

### 1. Platform-Specific File Separation

FMMLoader26 now automatically detects and handles platform-specific mod files. When creating a mod manifest, the system will detect folders named `windows`, `macos`, or `linux` and automatically tag files within these folders with the appropriate platform.

#### How It Works

When you import a mod with platform-specific folders:

```
MyMod/
├── windows/
│   └── ui-test.bundle
├── macos/
│   └── ui-test.bundle
└── linux/
    └── ui-test.bundle
```

The generated `manifest.json` will include platform tags:

```json
{
  "name": "MyMod",
  "files": [
    {
      "source": "windows/ui-test.bundle",
      "target_subpath": "ui-test.bundle",
      "platform": "windows"
    },
    {
      "source": "macos/ui-test.bundle",
      "target_subpath": "ui-test.bundle",
      "platform": "macos"
    },
    {
      "source": "linux/ui-test.bundle",
      "target_subpath": "ui-test.bundle",
      "platform": "linux"
    }
  ]
}
```

#### Installation Behavior

During mod installation:
- Only files matching the current platform (or files without platform tags) are installed
- Platform-specific folders are automatically stripped from the target path
- This ensures cross-platform mods work correctly on each operating system

### 2. Skins Mod Type Placement

The `skins` mod type now correctly installs to the game's bundle folder instead of the user's skins folder.

#### Before
```
skins mod → %USERPROFILE%/Documents/Sports Interactive/Football Manager 2026/skins/
```

#### After
```
skins mod → Game Installation/fm_Data/StreamingAssets/aa/StandaloneWindows64/
```

This ensures skins mods are placed in the standard bundle location where the game expects them.

### 3. Manifest Validation

When generating a manifest, the system now validates and logs warnings for missing fields:

- **author**: Logged as warning if empty
- **description**: Logged as warning if empty  
- **compatibility.fm_version**: Logged as warning if empty

These warnings help modders ensure their manifests are complete and provide users with proper information.

## For Mod Creators

### Creating Platform-Specific Mods

To create a mod that works across platforms but has platform-specific files:

1. **Structure your mod with platform folders:**
   ```
   YourMod/
   ├── windows/
   │   └── [Windows-specific files]
   ├── macos/
   │   └── [macOS-specific files]
   └── linux/
       └── [Linux-specific files]
   ```

2. **Import the mod as usual** - FMMLoader26 will automatically:
   - Detect the platform folders
   - Tag files appropriately
   - Generate a correct manifest

3. **Distribution** - Users on different platforms can use the same mod package, and only relevant files will be installed on their system.

### Creating Skins Mods

When creating skins mods:

1. Set `mod_type` to `"skins"` in your manifest
2. FMMLoader26 will automatically place files in the correct bundle folder
3. Users won't need to manually move files to the right location

### Example Manifest for Platform-Specific Mod

```json
{
  "name": "Cross-Platform UI Mod",
  "version": "1.0.0",
  "mod_type": "ui",
  "author": "Your Name",
  "description": "A UI mod that works on all platforms",
  "files": [
    {
      "source": "windows/ui-stadium.bundle",
      "target_subpath": "ui-stadium.bundle",
      "platform": "windows"
    },
    {
      "source": "macos/ui-stadium.bundle",
      "target_subpath": "ui-stadium.bundle",
      "platform": "macos"
    },
    {
      "source": "linux/ui-stadium.bundle",
      "target_subpath": "ui-stadium.bundle",
      "platform": "linux"
    },
    {
      "source": "shared/common.xml",
      "target_subpath": "shared/common.xml",
      "platform": null
    }
  ],
  "compatibility": {
    "fm_version": "26.0.0"
  }
}
```

## Technical Details

### Platform Detection Algorithm

1. First pass: Check if mod contains folders named `windows`, `macos`, or `linux` (case-insensitive)
2. If platform folders exist:
   - Files within these folders get tagged with the platform name
   - Target paths have the platform folder stripped (e.g., `windows/file.bundle` → `file.bundle`)
3. If no platform folders exist:
   - All files are tagged with `platform: null` (install on all platforms)

### Installation Filtering

During installation (`install_mod`):
```rust
// Pseudo-code
for each file in manifest.files:
    if file.platform is set:
        if file.platform != current_platform:
            skip this file
    install file to target location
```

### Mod Type Routing

The `get_target_for_type` function now routes mod types:
- `ui`, `bundle`, `skins` → Game bundle folder
- `tactics` → User tactics folder
- `graphics` → User graphics folder
- `editor-data` → User editor data folder

## Backward Compatibility

All changes maintain backward compatibility:
- Existing mods without platform folders continue to work as before
- Mods with `platform: null` install on all platforms
- The manifest structure supports both old and new formats

## Testing

Comprehensive unit tests verify:
1. Platform detection from folder names
2. Platform prefix removal from paths
3. Current platform identification
4. Correct target folder selection for each mod type
5. Platform filtering during installation

Run tests with:
```bash
cd src-tauri
cargo test
```

## Migration Guide

### For Users
No action needed - existing mods continue to work, and new features are automatic.

### For Mod Creators
To take advantage of platform-specific support:

1. **Optional**: Reorganize your mod with platform folders if you have platform-specific files
2. **Optional**: Update manifest to include platform tags for files
3. **Recommended**: Include `author`, `description`, and `compatibility.fm_version` in your manifest to avoid warnings

Example migration:
```
Before:
MyMod/
└── ui-test.bundle (Windows only)

After:
MyMod/
├── windows/
│   └── ui-test.bundle
├── macos/
│   └── ui-test.bundle
└── linux/
    └── ui-test.bundle
```

## Future Enhancements

Potential future improvements:
- Support for additional platform identifiers
- Validation of platform-specific files at manifest creation
- UI indicators for cross-platform mod compatibility
- Platform-specific dependency resolution
