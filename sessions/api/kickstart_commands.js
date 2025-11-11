#!/usr/bin/env node

// ==== IMPORTS ===== //

// ===== STDLIB ===== //
const fs = require('fs');
const path = require('path');
//--//

// ===== LOCAL ===== //
const { loadState, editState, loadConfig, PROJECT_ROOT, Mode } = require('../hooks/shared_state.js');
//--//

//-#

// ==== GLOBALS ===== //
const CONFIG = loadConfig();
const STATE = loadState();

//-#

// ==== FUNCTIONS ===== //

function formatConfigForDisplay(config) {
    /**Format config as readable markdown for kickstart display.*/
    return `**Current Configuration:**

**Trigger Phrases:**
- Implementation mode: ${config.trigger_phrases.implementation_mode}
- Discussion mode: ${config.trigger_phrases.discussion_mode}
- Task creation: ${config.trigger_phrases.task_creation}
- Task startup: ${config.trigger_phrases.task_startup}
- Task completion: ${config.trigger_phrases.task_completion}
- Context compaction: ${config.trigger_phrases.context_compaction}

**Git Preferences:**
- Default branch: ${config.git_preferences.default_branch}
- Has submodules: ${config.git_preferences.has_submodules}
- Add pattern: ${config.git_preferences.add_pattern}

**Environment:**
- Developer name: ${config.environment.developer_name}
- Project root: ${config.environment.project_root}`;
}

function loadProtocolFile(relativePath) {
    /**Load protocol markdown from protocols directory.*/
    const protocolPath = path.join(PROJECT_ROOT, 'sessions', 'protocols', relativePath);
    if (!fs.existsSync(protocolPath)) {
        return `Error: Protocol file not found: ${relativePath}`;
    }
    return fs.readFileSync(protocolPath, 'utf8');
}

// (No programmatic post-processing of protocol content)

function handleKickstartCommand(args, jsonOutput = false, fromSlash = false) {
    /**
     * Handle kickstart-specific commands for onboarding flow.
     *
     * Usage:
     *     kickstart full          - Initialize full kickstart onboarding
     *     kickstart subagents     - Initialize subagents-only onboarding
     *     kickstart next          - Load next module chunk
     *     kickstart complete      - Exit kickstart mode
     */
    if (!args || args.length === 0) {
        return formatKickstartHelp(jsonOutput);
    }

    const command = args[0].toLowerCase();
    const commandArgs = args.slice(1);

    if (command === 'full' || command === 'subagents') {
        return beginKickstart(command, jsonOutput);
    } else if (command === 'next') {
        return loadNextModule(jsonOutput);
    } else if (command === 'complete') {
        return completeKickstart(jsonOutput);
    } else {
        const errorMsg = `Unknown kickstart command: ${command}`;
        if (jsonOutput) {
            return { error: errorMsg };
        }
        return errorMsg;
    }
}

function formatKickstartHelp(jsonOutput) {
    /**Format help for kickstart commands.*/
    const commands = {
        'full': 'Initialize full kickstart onboarding',
        'subagents': 'Initialize subagents-only onboarding',
        'next': 'Load next module chunk based on current progress',
        'complete': 'Exit kickstart mode and clean up files'
    };

    if (jsonOutput) {
        return { available_commands: commands };
    }

    const lines = ['Kickstart Commands:'];
    for (const [cmd, desc] of Object.entries(commands)) {
        lines.push(`  ${cmd}: ${desc}`);
    }
    return lines.join('\n');
}

function loadNextModule(jsonOutput = false) {
    /**Load next module chunk based on current progress.*/
    const kickstartMeta = STATE.metadata?.kickstart;

    if (!kickstartMeta) {
        const errorMsg = 'Error: No kickstart metadata found. This is a bug.';
        if (jsonOutput) {
            return { error: errorMsg };
        }
        return errorMsg;
    }

    const sequence = kickstartMeta.sequence;
    const currentIndex = kickstartMeta.current_index;
    const completed = kickstartMeta.completed || [];

    if (!sequence) {
        const errorMsg = 'Error: No kickstart sequence found. This is a bug.';
        if (jsonOutput) {
            return { error: errorMsg };
        }
        return errorMsg;
    }

    // Mark current as completed
    const currentFile = sequence[currentIndex];

    // Move to next
    const nextIndex = currentIndex + 1;

    // Check if we've reached the end
    if (nextIndex >= sequence.length) {
        return completeKickstart(jsonOutput);
    }

    const nextFile = sequence[nextIndex];

    // Update state
    editState(s => {
        s.metadata.kickstart.current_index = nextIndex;
        s.metadata.kickstart.completed.push(currentFile);
        s.metadata.kickstart.last_active = new Date().toISOString();
        return s;
    });

    // Load next protocol
    const protocolContent = loadProtocolFile(`kickstart/${nextFile}`);

    if (jsonOutput) {
        return {
            success: true,
            next_file: nextFile,
            protocol: protocolContent
        };
    }

    return protocolContent;
}

const FULL_MODE_SEQUENCE = [
    '01-discussion.md',
    '02-implementation.md',
    '03-tasks-overview.md',
    '04-task-creation.md',
    '05-task-startup.md',
    '06-task-completion.md',
    '07-compaction.md',
    '08-agents.md',
    '09-api.md',
    '10-advanced.md',
    '11-graduation.md'
];

const SUBAGENTS_MODE_SEQUENCE = [
    '01-agents-only.md'
];

function beginKickstart(mode, jsonOutput = false) {
    /** Initialize kickstart sequence (full or subagents) and return first module content. */
    const sequence = (mode === 'subagents') ? SUBAGENTS_MODE_SEQUENCE : FULL_MODE_SEQUENCE;
    editState(s => {
        if (!s.metadata) s.metadata = {};
        s.metadata.kickstart = {
            mode: (mode === 'subagents') ? 'subagents' : 'full',
            sequence,
            current_index: 0,
            completed: [],
            last_active: new Date().toISOString(),
        };
        return s;
    });
    const firstFile = sequence[0];
    const protocolContent = loadProtocolFile(`kickstart/${firstFile}`);
    if (jsonOutput) return { success: true, started: mode, first_file: firstFile, protocol: protocolContent };
    return protocolContent;
}

function completeKickstart(jsonOutput = false) {
    /**Exit kickstart mode and clean up files/settings programmatically.*/
    // Switch to implementation mode if in discussion mode
    if (STATE.mode === Mode.NO) {
        editState(s => { s.mode = Mode.GO; return s; });
    }

    const sessionsDir = path.join(PROJECT_ROOT, 'sessions');

    // 1) Update settings.json to replace kickstart hook with regular session_start
    const settingsPath = path.join(PROJECT_ROOT, '.claude', 'settings.json');
    let settings = {};
    let updatedSettings = false;
    if (fs.existsSync(settingsPath)) {
        try { settings = JSON.parse(fs.readFileSync(settingsPath, 'utf8')); } catch { settings = {}; }
    }
    const hooksRoot = (settings && settings.hooks) || {};
    const sessionStartCfgs = (hooksRoot && hooksRoot.SessionStart) || [];
    for (const cfg of sessionStartCfgs) {
        const hooksList = (cfg && cfg.hooks) || [];
        for (const hook of hooksList) {
            const cmd = hook && hook.command;
            if (typeof cmd === 'string' && cmd.includes('kickstart_session_start')) {
                if (cmd.includes('kickstart_session_start.js')) {
                    hook.command = cmd.replace('kickstart_session_start.js', 'session_start.js');
                    updatedSettings = true;
                } else if (cmd.includes('kickstart_session_start.py')) {
                    hook.command = cmd.replace('kickstart_session_start.py', 'session_start.py');
                    updatedSettings = true;
                }
            }
        }
    }
    // De-duplicate
    const seen = new Set();
    const newCfgs = [];
    for (const cfg of sessionStartCfgs) {
        const hooksList = (cfg && cfg.hooks) || [];
        const newHooks = [];
        for (const hook of hooksList) {
            const cmd = hook && hook.command;
            if (typeof cmd === 'string') {
                if (seen.has(cmd)) { updatedSettings = true; continue; }
                seen.add(cmd);
            }
            newHooks.push(hook);
        }
        if (newHooks.length) newCfgs.push({ ...cfg, hooks: newHooks });
    }
    if (hooksRoot && typeof hooksRoot === 'object') hooksRoot.SessionStart = newCfgs;
    if (updatedSettings) {
        try { fs.mkdirSync(path.dirname(settingsPath), { recursive: true }); fs.writeFileSync(settingsPath, JSON.stringify(settings, null, 2), 'utf8'); } catch {}
    }

    // 2) Delete kickstart hook (JS variant)
    const jsHook = path.join(sessionsDir, 'hooks', 'kickstart_session_start.js');
    if (fs.existsSync(jsHook)) {
        try { fs.unlinkSync(jsHook); } catch {}
    }

    // 3) Delete kickstart protocols directory
    const protocolsDir = path.join(sessionsDir, 'protocols', 'kickstart');
    if (fs.existsSync(protocolsDir)) {
        try { fs.rmSync(protocolsDir, { recursive: true, force: true }); } catch {}
    }

    // 4) Delete kickstart setup task (check both locations)
    let taskFile = path.join(sessionsDir, 'tasks', 'h-kickstart-setup.md');
    if (!fs.existsSync(taskFile)) taskFile = path.join(sessionsDir, 'tasks', 'done', 'h-kickstart-setup.md');
    if (fs.existsSync(taskFile)) {
        try { fs.unlinkSync(taskFile); } catch {}
    }

    // 5) Clear kickstart metadata
    editState(s => { if (s.metadata) { delete s.metadata.kickstart; } return s; });

    // 6) Remove kickstart API (this file)
    const jsApi = path.join(sessionsDir, 'api', 'kickstart_commands.js');
    if (fs.existsSync(jsApi)) {
        try { fs.unlinkSync(jsApi); } catch {}
    }

    const successMessage = 'Kickstart complete! Cleanup finished and SessionStart restored.';
    if (jsonOutput) return { success: true, message: successMessage };
    return successMessage;
}

//-#

// ==== EXPORTS ===== //
module.exports = {
    handleKickstartCommand
};
//-#
