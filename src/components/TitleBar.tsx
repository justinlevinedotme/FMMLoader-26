import { getCurrentWindow } from '@tauri-apps/api/window';
import { useState } from 'react';

export function TitleBar() {
  const [isHovered, setIsHovered] = useState(false);
  const appWindow = getCurrentWindow();

  const handleClose = () => {
    void appWindow.close();
  };

  const handleMinimize = () => {
    void appWindow.minimize();
  };

  const handleMaximize = () => {
    void appWindow.toggleMaximize();
  };

  return (
    <div
      className="fixed top-0 left-0 right-0 h-12 flex items-center justify-between px-4 z-50 select-none pointer-events-none"
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      {/* Stoplight buttons - not draggable */}
      <div className="flex items-center gap-2 relative z-10 pointer-events-auto">
        <button
          onClick={handleClose}
          className="w-3 h-3 rounded-full bg-[#ff5f57] hover:bg-[#ff5f57]/80 flex items-center justify-center group"
          aria-label="Close"
        >
          {isHovered && (
            <svg
              className="w-2 h-2 text-[#4d0000] opacity-0 group-hover:opacity-100"
              viewBox="0 0 12 12"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <path d="M3 3l6 6M9 3l-6 6" />
            </svg>
          )}
        </button>
        <button
          onClick={handleMinimize}
          className="w-3 h-3 rounded-full bg-[#febc2e] hover:bg-[#febc2e]/80 flex items-center justify-center group"
          aria-label="Minimize"
        >
          {isHovered && (
            <svg
              className="w-2 h-2 text-[#975500] opacity-0 group-hover:opacity-100"
              viewBox="0 0 12 12"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <path d="M3 6h6" />
            </svg>
          )}
        </button>
        <button
          onClick={handleMaximize}
          className="w-3 h-3 rounded-full bg-[#28c840] hover:bg-[#28c840]/80 flex items-center justify-center group"
          aria-label="Maximize"
        >
          {isHovered && (
            <svg
              className="w-2 h-2 text-[#006500] opacity-0 group-hover:opacity-100"
              viewBox="0 0 12 12"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <path d="M3 5l2 2 4-4" />
            </svg>
          )}
        </button>
      </div>

      {/* Title - draggable */}
      <div
        data-tauri-drag-region
        className="absolute left-0 right-0 top-0 bottom-0 flex items-center justify-center cursor-grabbing pointer-events-auto"
      >
        <div className="text-sm font-medium text-foreground/70 ">FMMLoader26</div>
      </div>

      {/* Spacer for balance */}
      <div className="w-20" />
    </div>
  );
}
