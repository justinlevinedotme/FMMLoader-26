import { Button } from "@/components/ui/button";
import { RefreshCw, Download, AlertTriangle } from "lucide-react";
import { useUpdater } from "@/hooks/useUpdater";

export function UpdateBanner() {
  const { status, downloadAndInstall } = useUpdater();

  // Don't show banner if no update is available, still checking, or there's an error
  if (!status.available || status.checking) {
    return null;
  }

  return (
    <div className="bg-gradient-to-r from-blue-600 to-blue-700 dark:from-blue-700 dark:to-blue-800 text-white px-4 py-3 flex items-center justify-between shadow-lg">
      <div className="flex items-center gap-3 flex-1">
        <Download className="h-5 w-5 flex-shrink-0" />
        <div className="flex-1">
          <p className="font-medium">
            Update Available: v{status.latestVersion}
          </p>
          <p className="text-sm text-blue-100">
            A new version is ready to install
          </p>
        </div>
      </div>

      <div className="flex items-center gap-2">
        {status.downloading && (
          <div className="flex items-center gap-2 mr-4">
            <div className="text-sm">
              Downloading: {status.downloadProgress}%
            </div>
            <div className="w-32 h-2 bg-blue-800/50 rounded-full overflow-hidden">
              <div
                className="h-full bg-white transition-all duration-300"
                style={{ width: `${status.downloadProgress}%` }}
              />
            </div>
          </div>
        )}

        {status.installing && (
          <div className="flex items-center gap-2 text-sm mr-4">
            <RefreshCw className="h-4 w-4 animate-spin" />
            Installing...
          </div>
        )}

        {status.error && (
          <div className="flex items-center gap-2 text-sm text-red-200 mr-4">
            <AlertTriangle className="h-4 w-4" />
            {status.error}
          </div>
        )}

        {!status.downloading && !status.installing && (
          <Button
            onClick={() => downloadAndInstall()}
            disabled={status.downloading || status.installing}
            variant="secondary"
            size="sm"
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
                Update Now
              </>
            )}
          </Button>
        )}
      </div>
    </div>
  );
}
