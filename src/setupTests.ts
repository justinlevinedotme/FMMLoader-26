import '@testing-library/jest-dom/vitest';

// Radix Select calls pointer capture helpers that jsdom doesn't provide
if (!Element.prototype.hasPointerCapture) {
  Element.prototype.hasPointerCapture = () => false;
}
if (!Element.prototype.releasePointerCapture) {
  Element.prototype.releasePointerCapture = () => {};
}
