// KCode native — runtime web fixes injected into the Kimi Code web UI.
(function () {
  'use strict';

  // ── Anti-flicker CSS ───────────────────────────────────────────────────────
  const style = document.createElement('style');
  style.textContent = `
    *, *::before, *::after {
      scroll-behavior: auto !important;
      overscroll-behavior: none !important;
    }
    html, body, #app, main, [role="main"], [role="log"],
    [class*="scroll" i], [class*="chat" i], [class*="message" i] {
      -webkit-overflow-scrolling: auto !important;
    }
  `;
  document.head.appendChild(style);

  // ── Normalize scroll calls to instant behavior ─────────────────────────────
  function normalizeScrollArg(arg) {
    if (arg && typeof arg === 'object' && arg.behavior === 'smooth') {
      arg.behavior = 'auto';
    }
    return arg;
  }

  const wrap = (target, name) => {
    const orig = target[name];
    if (typeof orig !== 'function') return;
    target[name] = function (...args) {
      if (args.length === 1) args[0] = normalizeScrollArg(args[0]);
      return orig.apply(this, args);
    };
  };

  wrap(window, 'scrollTo');
  wrap(window, 'scrollBy');
  if (window.Element) {
    wrap(Element.prototype, 'scrollTo');
    wrap(Element.prototype, 'scrollBy');
  }

  // ── Auto-grant notification permission ─────────────────────────────────────
  if (window.Notification) {
    try {
      Object.defineProperty(Notification, 'permission', {
        get: () => 'granted',
        configurable: true,
      });
      Notification.requestPermission = function () {
        return Promise.resolve('granted');
      };
    } catch (err) {
      if (Notification.permission === 'default') {
        Notification.requestPermission().catch(() => {});
      }
    }
  }
})();
