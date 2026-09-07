// KCode native — runtime web fixes injected into the Kimi Code web UI.
(function () {
  'use strict';

  function setRootBackground() {
    if (document.documentElement) {
      document.documentElement.style.backgroundColor = '#0d0f12';
    }
    if (document.body) {
      document.body.style.backgroundColor = '#0d0f12';
    }
  }

  function injectStyles() {
    if (document.getElementById('kcode-native-styles')) return;

    const style = document.createElement('style');
    style.id = 'kcode-native-styles';
    style.textContent = `
      html, body { background-color: #0d0f12 !important; }
      *, *::before, *::after {
        scroll-behavior: auto !important;
        overscroll-behavior: none !important;
      }
      html, body, #app, main, [role="main"], [role="log"],
      [class*="scroll" i], [class*="chat" i], [class*="message" i] {
        -webkit-overflow-scrolling: auto !important;
      }

      /* macOS overlay titlebar: keep the sidebar brand header at its natural
         top position and center the Kimi Code brand; since it is centered it
         naturally clears the left traffic-light buttons. */
      .side .ch {
        padding-left: 40px !important;
        -webkit-app-region: drag !important;
      }
      .side .ch .ch-brand {
        justify-content: center !important;
      }
      .side .ch button,
      .side .ch input {
        -webkit-app-region: no-drag !important;
      }
    `;

    const target = document.head || document.documentElement;
    if (target) {
      target.appendChild(style);
    }
  }

  // Apply as early as possible, then guarantee styles are injected once the
  // DOM (and therefore <head>) is available.
  setRootBackground();

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', function onLoad() {
      document.removeEventListener('DOMContentLoaded', onLoad);
      setRootBackground();
      injectStyles();
    });
  } else {
    setRootBackground();
    injectStyles();
  }

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
