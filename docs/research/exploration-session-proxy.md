# Session Proxy: Technical Exploration

Date: 2026-02-02

## The Core Concept

Don't fork a browser. Build a **session proxy layer** that wraps any browser with isolated identity contexts. Each project gets isolated cookies, localStorage, auth tokens that can be injected into Chrome/Arc/whatever.

This document explores how to make this real.

---

## 1. How Identity Context Isolation Works Technically

### What Needs to Be Isolated

| Storage Type | Scope | Persistence | Isolation Mechanism |
|-------------|-------|-------------|---------------------|
| Cookies | Per domain, path, attributes | Session or persistent | Cookie store ID |
| localStorage | Per origin | Persistent | Origin key |
| sessionStorage | Per origin, per tab | Session only | Tab + origin |
| IndexedDB | Per origin | Persistent | Origin key |
| Cache API | Per origin | Persistent | Origin key |
| Credentials (HTTP Auth) | Per origin | Session | Auth cache |
| Service Workers | Per origin | Persistent | Registration scope |

### Existing Isolation Models

**Firefox Contextual Identities (Container Tabs)**
- Each container has a unique `cookieStoreId`
- All storage is keyed by this ID
- Extensions can create/manage containers via `contextualIdentities` API
- Tabs can be assigned to containers via `tabs.create({ cookieStoreId })`
- Full isolation: cookies, localStorage, IndexedDB, cache all separated

**Chrome Browser Profiles**
- Each profile is a separate directory (`~/.config/google-chrome/Profile N`)
- Complete storage isolation (like running separate Chrome instances)
- No cross-profile sharing
- Heavy: each profile has its own process tree
- No API for programmatic profile switching within a session

**Playwright/Puppeteer Browser Contexts**
- `browser.newContext()` creates isolated session
- Each context has separate cookies, localStorage, credentials
- Can save/restore state via `context.storageState()`
- Designed for automation, not human use

**Electron Session Partitions**
- `partition: 'persist:project-a'` creates named session
- Full isolation: cookies, cache, localStorage, IndexedDB
- API access to session properties (cookies, webRequest, etc.)
- Designed for app development, not browser replacement

### The Fundamental Insight

All browsers already implement session isolation internally. The question is: **how do we expose this to project-level control?**

---

## 2. Proxy Architecture Options

### Option A: HTTP/HTTPS Proxy (mitmproxy-style)

```
Browser <---> Session Proxy <---> Internet
              (localhost:8080)
```

**How it works:**
1. Proxy intercepts all HTTP/HTTPS traffic
2. Before forwarding request, inject cookies from project's cookie store
3. After receiving response, capture Set-Cookie headers to project's store
4. Browser's internal cookie jar is bypassed/ignored

**Pros:**
- Works with any browser
- Can modify any HTTP header
- Full visibility into traffic
- Can work with curl, wget, other tools

**Cons:**
- HTTPS requires MITM certificate (security UX nightmare)
- Doesn't isolate localStorage, IndexedDB, Cache API
- Service Workers break (different origin)
- WebSocket handling is complex
- Browser may cache aggressively, defeating isolation

**Verdict:** Too limited. Only solves cookies, creates more problems than it solves.

### Option B: Browser Extension

```
Browser <---> Extension (in-process) <---> Extension Backend
```

**How it works:**
1. Extension intercepts requests via `webRequest` API
2. Modifies Cookie headers before sending
3. Captures Set-Cookie from responses
4. Uses `browser.storage` for localStorage proxying
5. Backend stores per-project state

**Chrome Manifest V3 constraints:**
- `webRequestBlocking` deprecated (can't modify headers synchronously)
- Must use `declarativeNetRequest` (rules declared upfront)
- Cookie modification requires pre-declared rules
- Dynamic project switching becomes complex

**Firefox advantages:**
- `contextualIdentities` API is exactly what we need
- Full `webRequestBlocking` support in Manifest V3
- Can create containers programmatically
- Tabs assigned to containers inherit isolation

**Pros:**
- No MITM certificates needed
- Can access all storage APIs
- Runs in browser context
- Can provide UI for context switching

**Cons:**
- Chrome severely limited by Manifest V3
- Different implementation per browser
- Can't easily share state with CLI tools
- Extension performance overhead

**Verdict:** Firefox extension is viable. Chrome extension is crippled.

### Option C: CDP (Chrome DevTools Protocol) Controller

```
CLI/Agent <---> CDP <---> Chrome
```

**How it works:**
1. Launch Chrome with remote debugging enabled
2. Connect via CDP WebSocket
3. Create BrowserContexts for each project
4. Set cookies/storage via CDP commands
5. Control tab creation, assign contexts

**Key CDP domains:**
- `Target.createBrowserContext()` - isolated context
- `Network.setCookies()` - inject cookies
- `Storage.setStorageItems()` - set localStorage
- `Page.navigate()` - control navigation

**Pros:**
- Full control over all storage types
- Can save/restore complete state
- Works with automation tools (Playwright, Puppeteer)
- Browser-native isolation (no hacks)

**Cons:**
- Requires Chrome launched with special flags
- Human use awkward (debug mode)
- No Arc/Brave/Edge compatibility without same setup
- Session state lives in memory (crashes lose state)

**Verdict:** Excellent for agent automation. Poor for human daily-driver use.

### Option D: Hybrid Architecture (Recommended)

```
                    +-----------------+
                    | Session Manager |
                    | (daemon)        |
                    +--------+--------+
                             |
        +--------------------+--------------------+
        |                    |                    |
+-------v-------+    +-------v-------+    +-------v-------+
| Firefox       |    | Chrome/CDP    |    | CLI Tools     |
| Extension     |    | (automation)  |    | (curl, etc.)  |
+---------------+    +---------------+    +---------------+
```

**Components:**

1. **Session Manager Daemon**
   - Stores project contexts (cookies, localStorage snapshots)
   - Provides API for context CRUD
   - Syncs state between components
   - Persists to disk (SQLite or JSON)

2. **Firefox Extension (human use)**
   - Uses native `contextualIdentities` API
   - One container per project
   - Syncs container state with daemon
   - Provides project switcher UI

3. **CDP Controller (agent use)**
   - Launches Chrome with debugging
   - Creates BrowserContext per project
   - Injects state from daemon
   - Reports state changes back

4. **CLI Integration**
   - `stead context project-a` sets active context
   - Exports cookies to curl/wget compatible format
   - Proxy mode for tools that support it

**Verdict:** This is the architecture. Different tools for different use cases, unified state.

---

## 3. OAuth Flows with Isolated Contexts

OAuth is the hard case. Here's how it would work:

### Standard OAuth Flow (Problem)

```
1. App redirects to: https://accounts.google.com/oauth?redirect_uri=http://localhost:3000/callback
2. User authenticates at Google
3. Google redirects to: http://localhost:3000/callback?code=xxx
4. App exchanges code for tokens
5. App stores tokens, user is authenticated
```

**With session proxy, step 5 is isolated to a project context.**

### Challenge: Localhost Redirect Conflicts

Multiple projects want `localhost:3000/callback`. Solutions:

**A. Port Namespacing**
- Project A: `localhost:3100/callback`
- Project B: `localhost:3200/callback`
- Session manager assigns port ranges per project
- Requires configuring OAuth apps with specific ports

**B. Path Namespacing**
- All projects: `localhost:3000/oauth/project-a/callback`
- Single OAuth app, dynamic routing
- Session manager routes based on path

**C. Subdomain Namespacing** (best)
- `project-a.localhost:3000/callback`
- Single port, clear separation
- Requires OAuth apps to allow wildcard subdomains
- Most OAuth providers support this

### Challenge: Shared Auth Providers

User wants to use same Google account for multiple projects, but with separate sessions.

**Solution: Cookie partitioning by project**

When user authenticates in Project A context:
1. OAuth flow completes
2. Tokens stored in Project A's token store
3. Google cookies captured to Project A's cookie store
4. Other projects don't see these cookies

If user authenticates in Project B:
1. Google sees "new" user (no cookies)
2. User authenticates again (or uses different account)
3. Tokens stored in Project B's token store

**Implication:** User may need to re-authenticate per project for some services. This is a feature, not a bug (isolation = separate identities).

### Challenge: OAuth State Tokens

OAuth uses `state` parameter to prevent CSRF. With multiple projects:

**Risk:** State token generated in Project A could be completed in Project B context.

**Mitigation:** Session manager validates that OAuth callback's state matches the project that initiated the flow.

```
1. Project A starts OAuth, state=abc123
2. Session manager records: abc123 -> Project A
3. Callback received with state=abc123
4. Session manager routes to Project A context
```

---

## 4. Agent Browser Automation

### How Agents Would Use This

**Scenario:** Claude Code agent needs to:
1. Log into client's staging environment
2. Test a feature
3. Take screenshots
4. Log out

**Without Session Proxy:**
- Agent uses shared browser or fresh profile
- Can't access client's saved auth
- Must re-authenticate (problem if 2FA)
- Might leak auth to other agents

**With Session Proxy:**

```typescript
// Agent code
const session = await stead.context.get('client-staging');

// Launch browser with this context
const browser = await playwright.chromium.launch();
const context = await browser.newContext({
  storageState: session.storageState,
});

// Already logged in, credentials injected
const page = await context.newPage();
await page.goto('https://staging.client.com/admin');

// Do work...

// Save any session changes back
const newState = await context.storageState();
await stead.context.update('client-staging', { storageState: newState });

await browser.close();
```

### Key Agent Use Cases

| Use Case | Session Proxy Feature |
|----------|----------------------|
| Authenticated scraping | Pre-inject cookies, skip login |
| Multi-account testing | Separate context per account |
| Parallel execution | Isolated contexts prevent collision |
| Long-running sessions | State persists across agent restarts |
| Human handoff | Agent prepares context, human takes over |

### Browserless-Style Integration

[Browserless.io](https://docs.browserless.io/baas/session-management/standard-sessions) provides a model:

```javascript
// Create a named session
const session = await browserless.sessions.create({
  name: 'client-staging',
  ttl: 86400000, // 24 hours
});

// Connect with session
const browser = await puppeteer.connect({
  browserWSEndpoint: `wss://chrome.browserless.io?token=XXX&sessionId=${session.id}`,
});

// Session persists cookies, storage between connections
```

Session Proxy would provide similar API but for local/self-hosted contexts.

---

## 5. Human Context Switching

### The UX Goal

Developer works on `project-a`:
- All tabs in browser are `project-a` context
- Terminal sessions tagged as `project-a`
- OAuth redirects route to `project-a`

Notification: "Agent finished task on `project-b`"

Developer switches:
- Browser tabs for `project-b` surface (or open fresh)
- Terminal context switches
- Different auth sessions active

**This requires coordination across tools.**

### Firefox Implementation (Cleanest Path)

Firefox Multi-Account Containers already do this:

1. **Container per project**
   - "project-a" container (blue)
   - "project-b" container (green)
   - Visual distinction in tabs

2. **Default container assignment**
   - `github.com/org-a/*` -> project-a container
   - `github.com/org-b/*` -> project-b container
   - URL rules auto-assign tabs

3. **Extension enhancement**
   - One-click "switch to project-a"
   - Opens new window with all project-a tabs
   - Or filters existing tabs to show only project-a

4. **Sync with Session Manager**
   - Container state synced to daemon
   - Agents can read/write same state
   - Survives browser restart

### Chrome/Arc Implementation (Harder)

Chrome lacks container API. Options:

**A. Multiple Profiles (Heavy)**
- Each project = Chrome profile
- Profile switcher in toolbar
- Complete isolation
- Heavy resource use

**B. Extension with Simulated Containers**
- Extension manages cookie stores in background
- Intercepts requests, swaps cookies
- Manifest V3 limitations hurt this
- Possible but fragile

**C. CDP Control**
- Session manager launches Chrome with debugging
- Manages BrowserContexts via CDP
- Provides UI overlay for switching
- Works but feels hacky for daily use

### Keyboard-Driven Switching

For power users:

```bash
# Terminal command
stead switch project-a

# Effects:
# 1. Sets active context in session manager
# 2. Signals Firefox extension to switch
# 3. Updates terminal prompt/env
# 4. Changes port routing
```

Integration with tools like Raycast/Alfred for instant switching.

---

## 6. Existing Tech to Leverage

### Directly Usable

| Technology | What We Get | Effort |
|-----------|-------------|--------|
| Firefox `contextualIdentities` | Native container support | Low |
| Playwright BrowserContext | Agent automation | Low |
| mitmproxy | HTTP-level proxying if needed | Medium |
| SQLite | Local state persistence | Low |
| Keychain/libsecret | Credential storage | Medium |

### As Reference Implementations

| Technology | What We Learn |
|-----------|---------------|
| [Firefox Multi-Account Containers](https://github.com/mozilla/multi-account-containers) | Container UX patterns |
| [Browserless Sessions API](https://docs.browserless.io/baas/session-management/standard-sessions) | Remote session management |
| [OAuth2 Proxy](https://oauth2-proxy.github.io/oauth2-proxy/) | Proxy auth patterns |
| [Electron session](https://www.electronjs.org/docs/latest/api/session) | Session partition model |

### Build vs Buy

| Component | Build | Buy/Use |
|----------|-------|---------|
| Session state storage | Build (simple) | - |
| Firefox extension | Build | Base on MAC source |
| CDP controller | Build | Use Playwright |
| OAuth routing | Build | Reference oauth2-proxy |

---

## 7. Hard Problems

### 7.1 Cross-Origin Leakage

**Problem:** Site A embeds iframe from Site B. Which context does Site B use?

**Current browser behavior:** Same context (third-party cookies, now being deprecated).

**Session proxy behavior:** Should probably follow embedding page's context. But:
- What if Site B is a shared service (analytics, auth)?
- What if user wants Site B to use its own identity?

**Mitigation:** Configurable per-domain rules. Default: inherit from top frame.

### 7.2 Service Worker Persistence

**Problem:** Service workers register per-origin and persist. With context switching:
- Project A visits `example.com`, registers SW
- Project B visits `example.com`, sees Project A's SW

**Mitigation options:**
- Clear service workers on context switch (breaks offline)
- Namespace origins (project-a.example.com via proxy)
- Accept the limitation (SWs are per-origin, period)

**Reality:** This is mostly a non-issue. Service workers serve cached content; they don't typically leak identity data.

### 7.3 Browser Storage Quotas

**Problem:** localStorage has ~5MB limit per origin. With virtual contexts:
- Project A uses 4MB on `github.com`
- Project B tries to use 3MB on `github.com`
- Quota exceeded?

**Reality:** Each context is a separate cookie store. In Firefox containers, each has its own quota. In CDP contexts, same. The proxy layer inherits browser's existing partitioning.

### 7.4 Certificate/Security Warnings

**Problem:** If using HTTP proxy approach, HTTPS requires MITM cert.

**User experience:** Constant security warnings or manual cert trust.

**Mitigation:** Don't use HTTP proxy approach for primary isolation. Use browser-native mechanisms (containers, CDP contexts). Reserve proxy for CLI tools that accept custom CAs.

### 7.5 Credential Manager Integration

**Problem:** Browser's saved passwords are per-profile, not per-context.

**Scenario:** User saves GitHub password. Which context gets it?

**Options:**
- Separate password managers per context (complex)
- Shared credential store, context selects which to use (confusing)
- Disable browser password manager, use external (1Password, etc.)

**Recommended:** External password manager. Browser's built-in is already a mess with multiple accounts.

### 7.6 Sync Conflicts

**Problem:** Human uses Firefox container. Agent uses CDP. Both modify same context.

**Scenario:**
1. Human logs into service in "project-a" container
2. Agent runs Playwright with "project-a" context snapshot
3. Agent's session expires, doesn't update state
4. Human's session now also appears expired

**Mitigation:**
- Locking: context is "checked out" to one consumer
- Event sourcing: changes are merged, conflicts resolved
- Last-write-wins: simple but lossy

**Recommended:** Locking for now. Add merge later if needed.

### 7.7 State Export/Import Format

**Problem:** No standard format for browser session state.

**Playwright format:**
```json
{
  "cookies": [...],
  "origins": [
    {
      "origin": "https://example.com",
      "localStorage": [{"name": "key", "value": "val"}]
    }
  ]
}
```

**Netscape cookie format:**
```
# Netscape HTTP Cookie File
.example.com	TRUE	/	FALSE	0	name	value
```

**Decision:** Use Playwright's `storageState` format as canonical. It's modern, includes localStorage, and has tooling support.

---

## 8. Minimum Viable Implementation

### Phase 1: Agent-Only (Week 1-2)

**Goal:** Agents can use isolated browser contexts.

**Deliverable:**
```typescript
// stead-session library
const session = new SteadSession('project-a');
await session.load(); // From disk

const context = await playwright.chromium.newContext({
  storageState: session.toPlaywrightState(),
});

// ... agent work ...

await session.update(await context.storageState());
await session.save(); // To disk
```

**Storage:** JSON files in `~/.stead/sessions/project-a.json`

**No daemon, no extension, no human UX.**

### Phase 2: Human UX (Week 3-4)

**Goal:** Humans can switch contexts in Firefox.

**Deliverable:**
- Firefox extension using `contextualIdentities`
- CLI command `stead switch project-a`
- Extension syncs with `~/.stead/sessions/`
- Visual indicator of current context

### Phase 3: Full Integration (Week 5-8)

**Goal:** Unified system for agents and humans.

**Deliverable:**
- Session manager daemon with API
- Locking/checkout system
- OAuth redirect handling
- Chrome CDP support (secondary)
- Documentation and examples

---

## 9. Open Questions for Decision

1. **Firefox-first or browser-agnostic?**
   - Firefox has the APIs we need
   - Chrome users are majority
   - Build Firefox first, add Chrome later?

2. **Daemon or library?**
   - Daemon: central state, complexity
   - Library: simpler, but sync harder
   - Start with library, add daemon when needed?

3. **Session file format?**
   - Playwright's `storageState` JSON
   - Custom format with more metadata
   - Database (SQLite)?

4. **Scope of isolation?**
   - Just cookies and localStorage?
   - Include IndexedDB, Cache API?
   - Include Service Workers?

5. **Integration with stead control room?**
   - How does session state appear in UI?
   - Can control room "see" what each context has?

---

## Sources

- [Firefox Contextual Identities API](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Work_with_contextual_identities)
- [Firefox Multi-Account Containers](https://github.com/mozilla/multi-account-containers)
- [Chrome Site Isolation Design](https://www.chromium.org/developers/design-documents/site-isolation/)
- [Playwright Browser Contexts](https://playwright.dev/docs/browser-contexts)
- [Browserless Sessions API](https://docs.browserless.io/baas/session-management/standard-sessions)
- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)
- [OAuth2 Proxy Session Storage](https://oauth2-proxy.github.io/oauth2-proxy/configuration/session_storage/)
- [Electron Session Module](https://www.electronjs.org/docs/latest/api/session)
- [mitmproxy Sticky Cookies](https://docs.mitmproxy.org/stable/overview/features/)
- [Chrome Manifest V3 webRequest](https://developer.chrome.com/docs/extensions/reference/api/webRequest)
- [browser-use](https://github.com/browser-use/browser-use)
