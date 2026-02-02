# GitHub Research: Browser Session Isolation

Research conducted 2026-02-02 for the stead project.

**Goal**: Find existing projects for browser session/identity isolation that could inform building a "session proxy layer" - per-project identity isolation without forking browsers.

---

## Summary

The landscape divides into:
1. **Antidetect browsers** - Full browser forks with fingerprint spoofing (overkill, but good architecture reference)
2. **Firefox Multi-Account Containers** - The gold standard for cookie isolation without full browser separation
3. **Playwright/Puppeteer BrowserContexts** - Programmatic isolation, exactly what we need for automation
4. **Proxy-based tools** - HTTP proxies with cookie management (closest to our "session proxy layer" concept)
5. **Docker-based RBI** - Remote browser isolation, heavy but proven

**Key insight**: Firefox Containers + Playwright BrowserContexts are the closest to what we want. A session proxy layer would essentially be bringing that isolation model to any browser through a proxy.

---

## Category 1: Antidetect Browsers (Open Source)

### Camoufox
**Repo**: https://github.com/daijro/camoufox

**What it does**: Anti-detect browser built on Firefox. Sandboxes Playwright's internal code, uses BrowserForge for realistic fingerprint generation.

**Useful for us**:
- Architecture for fingerprint injection
- How they isolate Playwright detection
- BrowserForge fingerprint generator could be reused

**Limitations**: Full browser fork, not a proxy layer.

---

### GeekezBrowser
**Repo**: https://github.com/EchoHS/GeekezBrowser

**What it does**: Electron + Puppeteer antidetect browser with Xray-core proxy integration. Designed for multi-account e-commerce management.

**Useful for us**:
- Extension isolation per profile
- Canvas noise implementation
- Xray-core integration pattern

**Limitations**: Electron-based, heavy. Not a session layer.

---

### Undetectable Fingerprint Browser
**Repo**: https://github.com/itbrowser-net/undetectable-fingerprint-browser

**What it does**: Open source Multilogin alternative. Chromium-based with Canvas/WebGL/User-Agent spoofing.

**Useful for us**:
- "Consistency Analysis Engine" - ensures spoofing fields don't conflict
- Selenium/Playwright/Puppeteer integration patterns

**Limitations**: Full browser, not proxy-based.

---

### brodev3/antidetect
**Repo**: https://github.com/brodev3/antidetect

**What it does**: Console app for managing browser profiles with anti-detection. Simple UI, profile-based data isolation.

**Useful for us**:
- Simpler codebase to study
- Profile management patterns
- Data isolation per profile

**Limitations**: Console-based, limited scope.

---

## Category 2: Browser Extensions (Cookie/Session Isolation)

### Firefox Multi-Account Containers (Mozilla Official)
**Repo**: https://github.com/mozilla/multi-account-containers

**What it does**: Color-coded container tabs with isolated cookies. Official Mozilla extension.

**Useful for us**:
- **Best reference for cookie isolation UX**
- Container assignment patterns
- Integration with Mozilla VPN for per-container networking

**Limitations**: Firefox-only. Can't use in Chrome/Safari. Not programmable.

**Key insight**: This is the UX we want to replicate at the proxy layer.

---

### FoxyProxy
**Repo**: https://github.com/foxyproxy/browser-extension

**What it does**: Per-tab proxy routing. URL pattern matching for automatic proxy selection.

**Useful for us**:
- Per-tab proxy assignment patterns
- WebExtensions API usage for proxy control
- URL pattern matching logic

**Limitations**: Just proxies, no cookie isolation.

---

### SmartProxy
**Repo**: https://github.com/salarcode/SmartProxy

**What it does**: Automatic proxy enabling based on URL patterns. One-click rule creation.

**Useful for us**:
- Pattern-based proxy routing
- Cross-browser (Firefox + Chrome)

**Limitations**: Per-domain, not per-project isolation.

---

### Cookie Quick Manager
**Repo**: https://github.com/ysard/cookie-quick-manager

**What it does**: Firefox extension for cookie management with container support. Copy cookies between containers.

**Useful for us**:
- Container cookie manipulation API usage
- First-Party Isolation support patterns

**Limitations**: Firefox-only, manual management.

---

## Category 3: Playwright/Puppeteer Ecosystem

### Playwright (Microsoft)
**Repo**: https://github.com/microsoft/playwright

**What it does**: Browser automation with first-class BrowserContext isolation.

**Useful for us**:
- **BrowserContext is exactly the isolation primitive we need**
- `storageState` API for persisting cookies/localStorage to disk
- Proxy per context (`browser.newContext({ proxy: {...} })`)
- Works with Chromium, Firefox, WebKit

**Key quote from docs**: "Browser context is equivalent to a brand new browser profile. Creating a new browser context only takes a handful of milliseconds."

**Limitations**: Requires running browser through Playwright. Not a proxy layer for existing browser usage.

---

### puppeteer-extra + stealth plugin
**Repo**: https://github.com/berstend/puppeteer-extra

**What it does**: Puppeteer plugin system. Stealth plugin has 17 evasion modules for anti-detection.

**Useful for us**:
- Evasion techniques catalog
- Plugin architecture pattern
- What fingerprinting to handle

**Limitations**: Puppeteer-only. Some evasions are outdated.

---

### puppeteer-with-fingerprints
**Repo**: https://github.com/bablosoft/puppeteer-with-fingerprints

**What it does**: Puppeteer wrapper with fingerprint replacement from real devices.

**Useful for us**:
- Fingerprint injection patterns
- Real device fingerprint database concept

**Limitations**: Windows-only. Proprietary fingerprint service.

---

## Category 4: HTTP Proxy Libraries

### node-http-proxy
**Repo**: https://github.com/http-party/node-http-proxy

**What it does**: Programmable HTTP proxy for Node.js. Supports websockets.

**Useful for us**:
- **Core building block for session proxy layer**
- `cookieDomainRewrite` for rewriting Set-Cookie domains
- Proxy table API for routing rules
- Cookie path rewriting

**Limitations**: No built-in session/context awareness. We'd build that on top.

---

### http-proxy-middleware
**Repo**: https://github.com/chimurai/http-proxy-middleware

**What it does**: Express/Next.js middleware wrapper for node-http-proxy.

**Useful for us**:
- Custom router functions (host, path, async)
- Integration patterns for web apps

**Limitations**: Same as node-http-proxy - no session context.

---

### configurable-http-proxy
**Repo**: https://github.com/jupyterhub/configurable-http-proxy

**What it does**: node-http-proxy with REST API for dynamic route configuration. Used by JupyterHub.

**Useful for us**:
- **REST API for managing proxy routes** - exactly what we need
- Dynamic reconfiguration without restart
- Token-based auth for API

**Limitations**: Designed for JupyterHub's use case, may need adaptation.

---

### mitmproxy
**Repo**: https://github.com/mitmproxy/mitmproxy

**What it does**: Interactive HTTPS proxy with Python scripting.

**Useful for us**:
- Sticky cookies feature (automatically add session cookies)
- Python addon system for custom logic
- Mature cookie manipulation APIs

**Limitations**:
- Sticky cookies are global, not per-client
- Reverse proxy mode has cookie isolation issues (per GitHub issue #6342)

**Key issue**: In multi-tenant mode, session cookies leak between clients. This is exactly the problem we need to solve.

---

## Category 5: Docker/Remote Browser Isolation

### Neko
**Repo**: https://github.com/m1k1o/neko

**What it does**: Virtual browser in Docker, streamed via WebRTC. Like Mightyapp.

**Useful for us**:
- Cookie persistence across sessions
- WebRTC streaming architecture
- "No state on host browser" - exactly the isolation we want

**Limitations**: Requires Docker. High latency. Overkill for local dev.

---

### BrowserBox
**Repo**: https://github.com/BrowserBox/BrowserBox

**What it does**: Enterprise RBI platform. 60fps streaming, DLP features, Tor support.

**Useful for us**:
- CLI tool (`bbx`) for management
- Multi-tenant architecture

**Limitations**: Enterprise-focused, heavy.

---

### browserless
**Repo**: https://github.com/browserless/browserless

**What it does**: Headless Chrome in Docker with REST API. Session recording/replay.

**Useful for us**:
- Chrome extension loading
- Session management API
- Residential proxy integration

**Limitations**: Focused on scraping/automation, not developer workflow.

---

### docker-selenium
**Repo**: https://github.com/SeleniumHQ/docker-selenium

**What it does**: Selenium Grid in Docker with Chrome, Firefox, Edge.

**Useful for us**:
- `SE_DRAIN_AFTER_SESSION_COUNT` for session lifecycle
- Automatic cleanup patterns

**Limitations**: Testing-focused, not for interactive browsing.

---

## Category 6: Multi-Session Browsers

### Multi-Session Browser
**Repo**: https://github.com/pavloniym/multi-session-browser

**What it does**: Electron + Vue.js browser with isolated tabs for multiple social accounts.

**Useful for us**:
- Simple codebase for multi-session patterns
- Each tab has own cookies/storage

**Limitations**: Electron-based, social-media focused.

---

### GoLogin (API)
**Repo**: https://github.com/gologinapp/gologin

**What it does**: REST API for GoLogin antidetect browser. Create profiles, manage fingerprints, launch browsers.

**Useful for us**:
- **Profile API design** - create, list, configure profiles
- Proxy-per-profile configuration
- Cookie export/import
- Headless mode support

**Limitations**: Requires GoLogin service. API-only, not the browser itself.

---

## Architecture Insights

### What exists today:

```
Full Browser Fork         Browser Extension        Proxy Layer
     (heavy)                (limited)              (flexible)
        |                       |                      |
   Camoufox               Firefox Containers      node-http-proxy
   GeekezBrowser          FoxyProxy               mitmproxy
   GoLogin                SmartProxy              (no session-aware
                                                   solution exists)
```

### The gap we're filling:

No one has built a **session-aware proxy layer** that:
1. Routes requests through different cookie jars based on project context
2. Works with ANY browser (no extension required)
3. Provides API for managing sessions programmatically
4. Persists session state to disk per-project

### Closest prior art:

1. **Playwright BrowserContext** - Has the right abstraction, but requires running browser through Playwright
2. **Firefox Containers** - Has the right UX, but Firefox-only and extension-based
3. **configurable-http-proxy** - Has the REST API, but no session/cookie awareness
4. **mitmproxy** - Has cookie manipulation, but sessions are global

---

## Recommended Approach

Build on **node-http-proxy** or **http-proxy-3** with:

1. **Session context from request headers** - Client passes `X-Project-Context: picalyze`
2. **Per-context cookie jars** - Isolated cookie storage per context
3. **REST API for session management** - Create/list/delete contexts, export/import cookies
4. **Automatic HTTPS interception** - Generate certs per-domain (like mitmproxy)
5. **Browser extension (optional)** - Auto-inject context header based on tab groups

### Alternative: Fork mitmproxy

Add per-client session support to mitmproxy's sticky cookies. Python ecosystem, mature codebase, but adds Python dependency.

---

## Next Steps

1. Prototype node-http-proxy with per-header cookie jars
2. Test HTTPS interception patterns
3. Design session persistence format (compatible with Playwright storageState?)
4. Evaluate browser extension requirements for auto-context injection

---

## References

- [Playwright BrowserContext docs](https://playwright.dev/docs/browser-contexts)
- [Firefox Multi-Account Containers](https://github.com/mozilla/multi-account-containers)
- [mitmproxy sticky cookies issue](https://github.com/mitmproxy/mitmproxy/discussions/6342)
- [node-http-proxy](https://github.com/http-party/node-http-proxy)
- [configurable-http-proxy](https://github.com/jupyterhub/configurable-http-proxy)
