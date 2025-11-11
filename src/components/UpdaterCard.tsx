import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  RefreshCw,
  Download,
  CheckCircle2,
  AlertTriangle,
} from "lucide-react";
import { useUpdater } from "@/hooks/useUpdater";

export function UpdaterCard() {
  const { status, checkForUpdates, downloadAndInstall } = useUpdater();
  const [showLogs, setShowLogs] = useState(false);

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex-1">
            <CardTitle className="text-lg">App Updater</CardTitle>
            <CardDescription className="mt-1">
              Check for and install application updates
            </CardDescription>
          </div>
          <div className="flex items-center gap-2">
            {status.checking ? (
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <RefreshCw className="h-4 w-4 animate-spin" />
                Checking...
              </div>
            ) : status.available ? (
              <div className="flex items-center gap-2 text-sm text-green-600 dark:text-green-400">
                <Download className="h-4 w-4" />
                Update Available
              </div>
            ) : status.error ? (
              <div className="flex items-center gap-2 text-sm text-red-600 dark:text-red-400">
                <AlertTriangle className="h-4 w-4" />
                Error
              </div>
            ) : (
              <div className="flex items-center gap-2 text-sm text-green-600 dark:text-green-400">
                <CheckCircle2 className="h-4 w-4" />
                Up to date
              </div>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="text-sm text-muted-foreground space-y-2">
          <div className="flex justify-between items-center">
            <span>Current Version:</span>
            <span className="font-mono font-medium">
              {status.currentVersion || "Unknown"}
            </span>
          </div>
          {status.available && status.latestVersion && (
            <div className="flex justify-between items-center">
              <span>Latest Version:</span>
              <span className="font-mono font-medium text-green-600 dark:text-green-400">
                {status.latestVersion}
              </span>
            </div>
          )}
        </div>

        {status.error && (
          <div className="p-3 rounded-md bg-red-50 dark:bg-red-950/30 border border-red-200 dark:border-red-800">
            <p className="text-sm text-red-600 dark:text-red-400">
              {status.error}
            </p>
          </div>
        )}

        {status.downloading && (
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span>Downloading...</span>
              <span>{status.downloadProgress}%</span>
            </div>
            <div className="w-full h-2 bg-muted rounded-full overflow-hidden">
              <div
                className="h-full bg-blue-600 dark:bg-blue-400 transition-all duration-300"
                style={{ width: `${status.downloadProgress}%` }}
              />
            </div>
          </div>
        )}

        {status.installing && (
          <div className="flex items-center gap-2 text-sm text-blue-600 dark:text-blue-400">
            <RefreshCw className="h-4 w-4 animate-spin" />
            Installing update...
          </div>
        )}

        <div className="flex gap-2">
          <Button
            onClick={() => checkForUpdates(true)}
            disabled={
              status.checking || status.downloading || status.installing
            }
          >
            {status.checking ? (
              <>
                <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                Checking...
              </>
            ) : (
              <>
                <RefreshCw className="mr-2 h-4 w-4" />
                Check for Updates
              </>
            )}
          </Button>

          {status.available && (
            <Button
              variant="default"
              onClick={() => downloadAndInstall()}
              disabled={status.downloading || status.installing}
            >
              {status.downloading ? (
                <>
                  <Download className="mr-2 h-4 w-4 animate-spin" />
                  Downloading...
                </>
              ) : status.installing ? (
                <>
                  <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                  Installing...
                </>
              ) : (
                <>
                  <Download className="mr-2 h-4 w-4" />
                  Download & Install
                </>
              )}
            </Button>
          )}

          <Button
            variant="outline"
            onClick={() => setShowLogs(!showLogs)}
          >
            {showLogs ? "Hide Logs" : "Show Logs"}
          </Button>
        </div>

        {showLogs && status.logs.length > 0 && (
          <div className="mt-4">
            <div className="text-sm font-medium mb-2">Update Logs:</div>
            <div className="h-48 rounded-md border bg-muted/30 p-3 overflow-auto">
              <div className="space-y-1">
                {status.logs.map((log, index) => (
                  <div
                    key={index}
                    className="text-xs font-mono text-muted-foreground"
                  >
                    {log}
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {!status.logs.length && showLogs && (
          <div className="text-sm text-muted-foreground text-center p-4">
            No logs yet. Click "Check for Updates" to start.
          </div>
        )}
      </CardContent>
    </Card>
  );
}
