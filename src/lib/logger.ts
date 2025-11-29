/**
 * Centralized logging utility with environment-based configuration
 *
 * Log Levels (in order of severity):
 * - ERROR: Critical errors that need immediate attention
 * - WARN: Warning conditions that should be investigated
 * - INFO: General informational messages
 * - DEBUG: Detailed debugging information
 *
 * Usage:
 *   logger.error('Failed to load mod', { error, modId });
 *   logger.warn('Deprecated feature used', { feature: 'oldAPI' });
 *   logger.info('User logged in', { userId });
 *   logger.debug('Cache hit', { key, value });
 */

type LogLevel = 'ERROR' | 'WARN' | 'INFO' | 'DEBUG';

interface LoggerConfig {
  level: LogLevel;
  enabled: boolean;
}

class Logger {
  private config: LoggerConfig;
  private readonly levels: Record<LogLevel, number> = {
    ERROR: 0,
    WARN: 1,
    INFO: 2,
    DEBUG: 3,
  };

  constructor() {
    this.config = this.getConfig();
  }

  private getConfig(): LoggerConfig {
    // Check for LOG_LEVEL environment variable
    const envLevel = import.meta.env.VITE_LOG_LEVEL as LogLevel | undefined;
    const isDevelopment = import.meta.env.DEV;

    // Default to INFO in production, DEBUG in development
    const defaultLevel: LogLevel = isDevelopment ? 'DEBUG' : 'INFO';
    const level = envLevel && this.isValidLogLevel(envLevel) ? envLevel : defaultLevel;

    return {
      level,
      enabled: true,
    };
  }

  private isValidLogLevel(level: string): level is LogLevel {
    return ['ERROR', 'WARN', 'INFO', 'DEBUG'].includes(level);
  }

  private shouldLog(level: LogLevel): boolean {
    if (!this.config.enabled) return false;
    return this.levels[level] <= this.levels[this.config.level];
  }

  private formatMessage(level: LogLevel, message: string, data?: unknown): string {
    const timestamp = new Date().toISOString();
    const prefix = `[${timestamp}] [${level}]`;

    if (data !== undefined) {
      return `${prefix} ${message} ${JSON.stringify(data, null, 2)}`;
    }

    return `${prefix} ${message}`;
  }

  /**
   * Log an error message (always logged unless logger is disabled)
   */
  error(message: string, data?: unknown): void {
    if (!this.shouldLog('ERROR')) return;

    const formattedMessage = this.formatMessage('ERROR', message, data);
    console.error(formattedMessage);
  }

  /**
   * Log a warning message
   */
  warn(message: string, data?: unknown): void {
    if (!this.shouldLog('WARN')) return;

    const formattedMessage = this.formatMessage('WARN', message, data);
    console.warn(formattedMessage);
  }

  /**
   * Log an informational message
   */
  info(message: string, data?: unknown): void {
    if (!this.shouldLog('INFO')) return;

    const formattedMessage = this.formatMessage('INFO', message, data);
    console.info(formattedMessage);
  }

  /**
   * Log a debug message (only in development by default)
   */
  debug(message: string, data?: unknown): void {
    if (!this.shouldLog('DEBUG')) return;

    const formattedMessage = this.formatMessage('DEBUG', message, data);
    console.debug(formattedMessage);
  }

  /**
   * Update logger configuration at runtime
   */
  setConfig(config: Partial<LoggerConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Get current logger configuration
   */
  getLogLevel(): LogLevel {
    return this.config.level;
  }
}

// Export singleton instance
export const logger = new Logger();

// Export types for external use
export type { LogLevel, LoggerConfig };
