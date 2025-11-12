import { useState, useEffect, useCallback } from 'react';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { tauriCommands } from './useTauri';

export interface UpdateStatus {
  checking: boolean;
  available: boolean;
  downloading: boolean;
  installing: boolean;
  error: string | null;
  currentVersion: string | null;
  latestVersion: string | null;
  downloadProgress: number;
  logs: string[];
}

export const useUpdater = () => {
  const [status, setStatus] = useState<UpdateStatus>({
    checking: false,
    available: false,
    downloading: false,
    installing: false,
    error: null,
    currentVersion: null,
    latestVersion: null,
    downloadProgress: 0,
    logs: [],
  });
  const [appVersion, setAppVersion] = useState<string>('');

  // Get app version on mount
  useEffect(() => {
    tauriCommands.getAppVersion().then(setAppVersion).catch(console.error);
  }, []);

  const addLog = useCallback((message: string) => {
    const timestamp = new Date().toLocaleTimeString();
    const logMessage = `[${timestamp}] ${message}`;
    console.log(`[Updater] ${logMessage}`);
    setStatus(prev => ({
      ...prev,
      logs: [...prev.logs, logMessage],
    }));
  }, []);

  // Helper to log critical update events to backend file logs.
  // This persists update events to platform-specific log files (e.g., ~/Library/Application Support/FMMLoader26/logs/)
  // with structured [UPDATE_*] prefixes for easy filtering and troubleshooting.
  const logToBackend = useCallback((
    eventType: string,
    message: string,
    latestVersion: string | null = null,
    details?: string
  ) => {
    if (!appVersion) return;
    tauriCommands.logUpdateEvent(
      eventType,
      appVersion,
      latestVersion,
      message,
      details
    ).catch(err => console.error('Failed to log update event to backend:', err));
  }, [appVersion]);

  const checkForUpdates = useCallback(async (manual = false) => {
    try {
      setStatus(prev => ({ ...prev, checking: true, error: null }));

      if (manual) {
        addLog('Manual update check initiated by user');
        logToBackend('CHECK', 'Manual update check initiated by user');
      } else {
        addLog('Automatic update check started');
        logToBackend('CHECK', 'Automatic update check started');
      }

      addLog('Checking updater endpoint: https://github.com/justinlevinedotme/FMMLoader-26/releases/latest/download/latest.json');

      const update = await check();

      if (update === null) {
        addLog('No update available - app is up to date');
        logToBackend('CHECK', 'No update available - app is up to date', appVersion);
        setStatus(prev => ({
          ...prev,
          checking: false,
          available: false,
        }));
        return null;
      }

      addLog(`Update found! Current version: ${update.currentVersion}, Latest version: ${update.version}`);
      addLog(`Release date: ${update.date}`);
      addLog(`Update body: ${update.body || 'No release notes available'}`);

      logToBackend(
        'FOUND',
        'Update available',
        update.version,
        `Release date: ${update.date}, Body: ${update.body || 'No release notes'}`
      );

      setStatus(prev => ({
        ...prev,
        checking: false,
        available: true,
        currentVersion: update.currentVersion,
        latestVersion: update.version,
      }));

      return update;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
      const errorDetails = error instanceof Error
        ? `${error.name}: ${error.message}\nStack: ${error.stack || 'No stack trace'}`
        : `Non-Error object: ${JSON.stringify(error)}`;

      console.error('[Updater] Full error details:', error);
      addLog(`Error checking for updates: ${errorMessage}`);
      logToBackend('ERROR', `Error checking for updates: ${errorMessage}`, null, errorDetails);
      setStatus(prev => ({
        ...prev,
        checking: false,
        error: errorMessage,
      }));
      return null;
    }
  }, [addLog, logToBackend]);

  const downloadAndInstall = useCallback(async () => {
    try {
      addLog('Starting update download and installation process');
      setStatus(prev => ({ ...prev, downloading: true, error: null }));

      const update = await check();

      if (update === null) {
        addLog('No update available to download');
        logToBackend('DOWNLOAD', 'No update available to download', status.latestVersion);
        setStatus(prev => ({ ...prev, downloading: false }));
        return false;
      }

      addLog(`Downloading update ${update.version}...`);
      logToBackend('DOWNLOAD', `Starting download of version ${update.version}`, update.version);

      let downloadedBytes = 0;
      let totalBytes = 0;
      let lastLoggedProgress = 0;

      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case 'Started':
            addLog(`Download started - Content length: ${event.data.contentLength || 'unknown'} bytes`);
            totalBytes = event.data.contentLength || 0;
            break;
          case 'Progress':
            downloadedBytes += event.data.chunkLength;
            const progress = totalBytes > 0
              ? Math.round((downloadedBytes / totalBytes) * 100)
              : 0;
            setStatus(prev => ({ ...prev, downloadProgress: progress }));
            addLog(`Download progress: ${downloadedBytes}/${totalBytes} bytes (${progress}%)`);

            // Log progress to backend at 25% increments to avoid spam
            if (progress >= lastLoggedProgress + 25 && progress > 0) {
              logToBackend('DOWNLOAD', `Download progress: ${progress}%`, update.version);
              lastLoggedProgress = progress;
            }
            break;
          case 'Finished':
            addLog('Download finished successfully');
            logToBackend('DOWNLOAD', 'Download finished successfully', update.version);
            setStatus(prev => ({ ...prev, downloading: false, installing: true }));
            break;
        }
      });

      addLog('Update installed successfully - restarting application...');
      logToBackend('INSTALL', 'Update installed successfully - restarting application', update.version);
      setStatus(prev => ({ ...prev, installing: false }));

      // Relaunch the app to apply the update
      setTimeout(() => {
        relaunch();
      }, 1000);

      return true;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
      const errorDetails = error instanceof Error
        ? `${error.name}: ${error.message}\nStack: ${error.stack || 'No stack trace'}`
        : `Non-Error object: ${JSON.stringify(error)}`;

      console.error('[Updater] Full error details:', error);
      addLog(`Error downloading/installing update: ${errorMessage}`);
      logToBackend('ERROR', `Error downloading/installing update: ${errorMessage}`, status.latestVersion, errorDetails);
      setStatus(prev => ({
        ...prev,
        downloading: false,
        installing: false,
        error: errorMessage,
      }));
      return false;
    }
  }, [addLog, logToBackend, status.latestVersion]);

  // Check for updates on mount
  useEffect(() => {
    addLog('Updater hook initialized');
    // Delay initial check to let the app fully load
    const timer = setTimeout(() => {
      checkForUpdates(false);
    }, 3000);
    return () => clearTimeout(timer);
  }, [checkForUpdates, addLog]);

  return {
    status,
    checkForUpdates,
    downloadAndInstall,
    addLog,
  };
};
