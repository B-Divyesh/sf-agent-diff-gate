import "./style.css";
import "./hero-image.css";

type Check = {
  label: string;
  detail: string;
  state: "ready" | "risk" | "missing" | "done";
};
type Draft = {
  id?: string;
  title: string;
  owner: string;
  changed: string[];
  checks: Check[];
  source: string;
  status?: string;
  source_url?: string;
  approved_by?: string;
  approved_at?: string;
};
type Auth = {
  authenticated: boolean;
  entra_sign_in_configured: boolean;
  github_app_configured: boolean;
  install_url?: string;
  user?: string;
  team?: string;
};
type AuditEntry = {
  id: string;
  actor: string;
  action: string;
  detail: string;
  created_at: string;
};
const DEMO_STORAGE = "demo:diff-gate";
const sample: Draft = {
  title: "Add organization-level retention controls",
  owner: "Mira Chen",
  source: "PR #482 · agent-authored",
  changed: [
    "src/policy/retention.ts",
    "db/migrations/20260828_retention.sql",
    "src/api/export.ts",
  ],
  checks: [
    {
      label: "Contract changed",
      detail: "Export API now returns retention state.",
      state: "ready",
    },
    {
      label: "Migration found",
      detail: "Adds retention_days and backfills 14,382 rows.",
      state: "risk",
    },
    {
      label: "Test evidence",
      detail: "pnpm test: 214 passed · migration check: passed",
      state: "ready",
    },
    {
      label: "Risky path",
      detail: "db/migrations requires database owner sign-off.",
      state: "risk",
    },
  ],
};
let demo =
  location.pathname === "/demo" ||
  new URLSearchParams(location.search).get("demo") === "1";
let draft: Draft | null = demo ? loadDemo() : null;
let auth: Auth = {
  authenticated: false,
  entra_sign_in_configured: false,
  github_app_configured: false,
};
let packetList: Array<{ id: string; title: string; status: string }> = [];
let auditHistory: AuditEntry[] = [];
let retentionDays = 90;
let offline = !navigator.onLine;
const app = document.querySelector<HTMLDivElement>("#app")!;
const esc = (s: string) =>
  s.replace(
    /[&<>'"]/g,
    (c) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", "'": "&#39;", '"': "&quot;" })[
        c
      ]!,
  );
function loadDemo() {
  try {
    const saved = sessionStorage.getItem(DEMO_STORAGE);
    if (saved) return JSON.parse(saved) as Draft;
  } catch {
    sessionStorage.removeItem(DEMO_STORAGE);
  }
  return structuredClone(sample);
}
function saveDemo() {
  if (demo && draft)
    sessionStorage.setItem(DEMO_STORAGE, JSON.stringify(draft));
}
function syncRoute() {
  const nextDemo =
    location.pathname === "/demo" ||
    new URLSearchParams(location.search).get("demo") === "1";
  if (nextDemo !== demo) {
    demo = nextDemo;
    draft = demo ? loadDemo() : null;
  }
  render();
  if (!demo && location.pathname === "/") void loadAuth();
}
function nav(path: string) {
  if (demo && path !== "/demo") sessionStorage.removeItem(DEMO_STORAGE);
  history.pushState({}, "", path);
  syncRoute();
}
async function api<T>(url: string, init?: RequestInit): Promise<T> {
  const response = await fetch(url, {
    ...init,
    headers: { "content-type": "application/json", ...(init?.headers || {}) },
  });
  if (!response.ok) {
    const body = await response
      .json()
      .catch(() => ({ error: "The request failed. Try again." }));
    throw new Error(body.error || "The request failed.");
  }
  if (response.status === 204) return undefined as T;
  return response.json() as Promise<T>;
}
async function loadAudit(id = draft?.id) {
  auditHistory = !demo && id ? await api<AuditEntry[]>(`/api/packets/${id}/audit`) : [];
}
async function loadAuth() {
  try {
    auth = await api<Auth>("/api/auth/status");
    if (auth.authenticated) {
      const [packets, settings] = await Promise.all([
        api<Array<{ id: string; title: string; status: string }>>("/api/packets"),
        api<{ retention_days: number }>("/api/settings"),
      ]);
      packetList = packets;
      retentionDays = settings.retention_days;
    } else {
      packetList = [];
    }
  } catch {
    auth = {
      authenticated: false,
      entra_sign_in_configured: false,
      github_app_configured: false,
    };
    packetList = [];
  } finally {
    render();
  }
}
function shell(content: string, page: string, title: string) {
  document.title = title;
  document.querySelector<HTMLLinkElement>('link[rel="canonical"]')!.href =
    `${location.origin}${location.pathname}`;
  app.innerHTML = `<a class="skip" href="#main">Skip to content</a><header class="site-head"><a class="wordmark" href="/" data-nav><span class="mark" aria-hidden="true">≡</span> Diff Gate</a><nav aria-label="Main navigation"><a href="/demo" data-nav>Demo</a><a href="/#how">How it works</a><a href="/privacy" data-nav>Privacy</a></nav></header>${demo ? `<aside class="demo-bar" aria-label="Demo mode"><span><b>Demo</b> — sample data, nothing is saved</span><span><button class="text-button" id="reset-demo">Reset demo</button><button class="text-button" id="start-real">Start for real</button></span></aside>` : ""}<p id="announcer" class="sr-only" aria-live="polite">${page}</p><main id="main" tabindex="-1">${content}</main><footer><span>Diff Gate makes change ownership visible.</span><span><a href="/privacy" data-nav>Privacy</a><a href="/terms" data-nav>Terms</a><span>Built by Param Factory</span></span><small>v0.3.0</small></footer>`;
  bind();
  requestAnimationFrame(() =>
    document.querySelector<HTMLElement>("h1")?.focus({ preventScroll: true }),
  );
}
function realStart() {
  if (!auth.authenticated)
    return `<section class="empty-panel" aria-labelledby="real-title"><p class="eyebrow">Team review</p><h2 id="real-title">Sign in before reviewing repository changes</h2><p>Sociobot Entra identifies the reviewer. Packets are visible only to that reviewer’s team.</p>${auth.entra_sign_in_configured ? '<a class="primary link-button" href="/auth/entra">Sign in with Sociobot</a>' : '<p class="status warning" role="status">Sociobot Entra sign-in is not configured on this deployment. The sample demo still works without an account.</p>'}</section>`;
  const history = packetList.length
    ? `<section class="packet-history" aria-labelledby="history-title"><h3 id="history-title">Saved review packets</h3><ul>${packetList.map((packet) => `<li><button class="history-link" data-packet="${esc(packet.id)}">${esc(packet.title)} <span>${esc(packet.status)}</span></button></li>`).join("")}</ul></section>`
    : '<p class="status">No saved review packets yet.</p>';
  return `<section class="empty-panel" aria-labelledby="real-title"><div class="signed-in"><p class="eyebrow">${esc(auth.team || "Team")} · signed in as ${esc(auth.user || "reviewer")}</p><button class="text-button" id="sign-out">Sign out</button></div><h2 id="real-title">Open a real review packet</h2><p>Import a pull request through the team-bound GitHub App, or record a packet by hand.</p><form id="import-form" class="packet-form"><label for="pr-url">GitHub pull request URL</label><div class="license-row"><input id="pr-url" type="url" required placeholder="https://github.com/owner/repo/pull/123"><button class="primary" type="submit">Import pull request</button></div>${auth.github_app_configured ? "" : '<p class="status warning">A GitHub App installation must be bound to this Sociobot team before importing.</p>'}${auth.install_url ? `<a href="${esc(auth.install_url)}" rel="external">Install the GitHub App (opens GitHub)</a>` : ""}</form><form id="blank-form" class="packet-form"><h3>Create a packet by hand</h3><label for="packet-title">Change title</label><input id="packet-title" required maxlength="180"><label for="packet-owner">Responsible owner</label><input id="packet-owner" value="${esc(auth.user || "")}" required><label for="packet-files">Changed files, one per line</label><textarea id="packet-files" rows="3"></textarea><label for="packet-tests">Test evidence</label><textarea id="packet-tests" rows="2" placeholder="Command and result"></textarea><button class="secondary" type="submit">Save review packet</button></form>${history}<form id="retention-form" class="packet-form"><h3>Data retention</h3><label for="retention-days">Delete packets and their audit history after this many days</label><div class="license-row"><input id="retention-days" type="number" min="1" max="3650" required value="${retentionDays}"><button class="secondary" type="submit">Save retention</button></div></form><p id="real-note" class="status" role="status"></p></section>`;
}
function packetUI() {
  if (!draft) return realStart();
  const blockers = draft.checks.filter(
    (c) => c.state === "risk" || c.state === "missing",
  ).length;
  const immutable = draft.status === "approved";
  const ownerBlocked = !demo && auth.user !== draft.owner;
  const history = !demo && auditHistory.length
    ? `<section class="audit" aria-labelledby="audit-title"><h3 id="audit-title">Audit history</h3><ol>${auditHistory.map((entry) => `<li><strong>${esc(entry.action.replaceAll("_", " "))}</strong><span>${esc(entry.actor)} · ${esc(new Date(entry.created_at).toLocaleString())}</span><p>${esc(entry.detail)}</p></li>`).join("")}</ol></section>`
    : "";
  return `<section class="packet" aria-labelledby="packet-title"><div class="packet-head"><div><p class="eyebrow">${esc(draft.source)}</p><h2 id="packet-title">${esc(draft.title)}</h2><p class="packet-meta">Accountable owner: <strong>${esc(draft.owner)}</strong>${draft.approved_by ? ` · Approved by <strong>${esc(draft.approved_by)}</strong>` : ""}</p>${draft.source_url ? `<p><a href="${esc(draft.source_url)}" rel="external">Open source pull request (opens GitHub)</a></p>` : ""}</div><div class="stamp ${blockers ? "hold" : "approved"}" aria-label="${immutable ? "Approved" : blockers ? "Review hold" : "Ready to approve"}"><span>${immutable ? "APPROVED" : blockers ? "HOLD" : "READY"}</span><small>${immutable ? "record retained" : blockers ? `${blockers} owner check${blockers === 1 ? "" : "s"}` : "evidence attached"}</small></div></div><div class="packet-grid"><section aria-labelledby="changed-title"><h3 id="changed-title">Changed files</h3><ul class="file-list">${draft.changed.length ? draft.changed.map((f, i) => `<li><span class="file-no">${String(i + 1).padStart(2, "0")}</span><code>${esc(f)}</code></li>`).join("") : "<li>No changed files recorded.</li>"}</ul></section><section aria-labelledby="checks-title"><h3 id="checks-title">Review evidence</h3><ul class="checks">${draft.checks.map((c, i) => `<li class="check ${c.state}"><span class="state-dot" aria-hidden="true">${c.state === "risk" ? "!" : c.state === "done" ? "✓" : "•"}</span><div><strong>${esc(c.label)}</strong><p>${esc(c.detail)}</p></div>${!immutable && (c.state === "risk" || c.state === "missing") ? `<button class="small-button" data-resolve="${i}">Mark reviewed</button>` : ""}</li>`).join("")}</ul></section></div>${history}<div class="packet-actions"><button class="secondary" id="export-packet">Export packet${demo ? "" : " and history"}</button>${demo ? "" : '<button class="danger-button" id="delete-packet">Delete packet</button>'}<button class="primary" id="approve-packet" ${blockers || immutable || ownerBlocked ? 'disabled aria-describedby="approval-note"' : ""}>${immutable ? "Approved" : "Approve for merge"}</button><p id="approval-note" class="status ${blockers || ownerBlocked ? "warning" : "success"}" role="status">${immutable ? "This approval is retained and immutable." : ownerBlocked ? "Only the named owner can approve this packet." : blockers ? "Resolve and save every flagged owner check before approval." : "All evidence is saved. Approval records the named owner."}</p></div></section>`;
}
function landing() {
  queueMicrotask(() => {
    const hero = document.querySelector(".hero-print");
    if (hero)
      hero.innerHTML =
        '<img src="/change-control.webp" width="900" height="600" fetchpriority="high" decoding="async" alt="Printed file sheets and review marks arranged across a change-control desk."><b class="art-stamp">CHECK</b>';
  });
  shell(
    `<section class="hero"><div class="hero-copy"><p class="eyebrow">Accountable review for agent changes</p><h1 tabindex="-1">Review agent changes before merge</h1><p class="lede">For small software teams who need an owner and evidence before an agent-made change lands.</p><div class="button-row"><button class="primary" id="hero-demo">Try it with sample data</button><span class="after-action">Opens a complete review packet.</span></div><ul class="facts"><li>Sample data stays in this browser.</li><li>Sociobot sign-in limits packets to one team.</li><li>GitHub imports read every changed-file page.</li></ul></div><figure class="hero-art"><div class="hero-print" role="img" aria-label="Printed file sheets, a test receipt, and an approval stamp arranged as a review desk."><i class="paper p1"></i><i class="paper p2"></i><i class="receipt"></i><b class="art-stamp">CHECK</b></div><figcaption>Every packet names an owner and records review evidence.</figcaption></figure></section><section class="live-area" aria-labelledby="desk-title"><div class="section-kicker"><p class="eyebrow">Live review desk</p><h2 id="desk-title">Find the merge blockers first</h2></div>${packetUI()}</section><section id="how" class="how" aria-labelledby="how-title"><p class="eyebrow">How it works</p><h2 id="how-title">Make the review decision visible</h2><ol><li><b>Sign in.</b><span>Sociobot Entra identifies the reviewer and team.</span></li><li><b>Import a pull request.</b><span>The team-bound GitHub App reads every changed-file page.</span></li><li><b>Record the decision.</b><span>Resolve evidence and retain the named approval.</span></li></ol></section><section class="boundary" aria-labelledby="boundary-title"><h2 id="boundary-title">It does not merge code for you</h2><p>Diff Gate keeps security findings advisory. Your team decides what to change and who approves it.</p></section>`,
    "Diff Gate home",
    "Diff Gate — Review agent changes before merge",
  );
}
function demoPage() {
  shell(
    `<section class="app-page"><div class="section-kicker"><p class="eyebrow">Sample review packet</p><h1 tabindex="-1">See an agent change under review</h1><p class="lede">Inspect the owner, evidence, and risky paths. This sample is separate from real packets.</p></div>${packetUI()}</section>`,
    "Demo — Diff Gate",
    "Demo — Diff Gate",
  );
}
function legal(kind: "privacy" | "terms") {
  const privacy = kind === "privacy";
  shell(
    `<article class="legal"><p class="eyebrow">Diff Gate ${privacy ? "privacy" : "terms"}</p><h1 tabindex="-1">${privacy ? "How Diff Gate handles review data" : "Terms for using Diff Gate"}</h1>${privacy ? "<p>Demo data stays in this browser. It is cleared when you leave demo mode.</p><p>Real packets contain the title, responsible owner, evidence, and GitHub paths that your signed-in team submits.</p><p>Your team chooses a retention period from 1 to 3,650 days. Expired packets and their audit history are deleted.</p><p>You can also delete a packet and its audit history immediately.</p><p>Diff Gate uses Sociobot Entra for identity. The team-bound GitHub App reads only pull requests your team can access.</p>" : "<p>Diff Gate records review evidence. Your team remains responsible for code, security findings, and merge decisions.</p><p>Use the service only with repositories you are allowed to review.</p>"}</article>`,
    `${privacy ? "Privacy" : "Terms"} — Diff Gate`,
    `${privacy ? "Privacy" : "Terms"} — Diff Gate`,
  );
}
function notFound() {
  shell(
    `<section class="not-found"><p class="eyebrow">404</p><h1 tabindex="-1">This review desk is empty</h1><p>That page does not exist. Return to the packet list to continue reviewing.</p><button class="primary" id="go-home">Go to Diff Gate</button></section>`,
    "Not found — Diff Gate",
    "Not found — Diff Gate",
  );
}
function render() {
  const path = location.pathname;
  if (path === "/") landing();
  else if (path === "/demo") demoPage();
  else if (path === "/privacy") legal("privacy");
  else if (path === "/terms") legal("terms");
  else notFound();
}
function createBlank() {
  draft = {
    title: "Untitled change",
    owner: "Add responsible owner",
    source: "New review packet",
    changed: [],
    checks: [
      {
        label: "Changed contract",
        detail: "Describe the public behavior that changed.",
        state: "missing",
      },
      {
        label: "Test evidence",
        detail: "Add the command and result before approval.",
        state: "missing",
      },
    ],
  };
  saveDemo();
  render();
}
async function saveManual(event: SubmitEvent) {
  event.preventDefault();
  const title =
    document.querySelector<HTMLInputElement>("#packet-title")!.value;
  const owner =
    document.querySelector<HTMLInputElement>("#packet-owner")!.value;
  const changed = document
    .querySelector<HTMLTextAreaElement>("#packet-files")!
    .value.split("\n")
    .map((v) => v.trim())
    .filter(Boolean);
  const evidence = document
    .querySelector<HTMLTextAreaElement>("#packet-tests")!
    .value.trim();
  const checks: Check[] = [
    {
      label: "Changed contract",
      detail: "Reviewer must confirm the public behavior that changed.",
      state: "missing",
    },
    {
      label: "Test evidence",
      detail: evidence || "Add a command and result before approval.",
      state: evidence ? "ready" : "missing",
    },
  ];
  try {
    const saved = await api<any>("/api/packets", {
      method: "POST",
      body: JSON.stringify({
        title,
        owner,
        data: { source: "Manual review packet", changed, checks },
      }),
    });
    draft = {
      ...JSON.parse(saved.data),
      id: saved.id,
      title: saved.title,
      owner: saved.owner,
      status: saved.status,
    };
    await loadAudit(saved.id);
    render();
  } catch (error) {
    const note = document.querySelector("#real-note")!;
    note.textContent =
      error instanceof Error ? error.message : "Could not save the packet.";
  }
}
async function importPr(event: SubmitEvent) {
  event.preventDefault();
  const url = document.querySelector<HTMLInputElement>("#pr-url")!.value;
  try {
    const saved = await api<any>("/api/github/import", {
      method: "POST",
      body: JSON.stringify({ pr_url: url }),
    });
    draft = {
      ...JSON.parse(saved.data),
      id: saved.id,
      title: saved.title,
      owner: saved.owner,
      status: saved.status,
      source_url: saved.source_url,
    };
    await loadAudit(saved.id);
    render();
  } catch (error) {
    const note = document.querySelector("#real-note")!;
    note.textContent =
      error instanceof Error
        ? error.message
        : "Could not import the pull request.";
  }
}
async function saveEvidence() {
  if (!draft || demo || !draft.id) return;
  const saved = await api<any>(`/api/packets/${draft.id}`, {
    method: "PUT",
    body: JSON.stringify({
      data: {
        source: draft.source,
        changed: draft.changed,
        checks: draft.checks,
      },
    }),
  });
  draft = {
    ...JSON.parse(saved.data),
    id: saved.id,
    title: saved.title,
    owner: saved.owner,
    status: saved.status,
    source_url: saved.source_url,
    approved_by: saved.approved_by,
    approved_at: saved.approved_at,
  };
  await loadAudit(saved.id);
}
function bind() {
  document.querySelectorAll<HTMLElement>("[data-nav]").forEach((a) =>
    a.addEventListener("click", (e) => {
      e.preventDefault();
      nav((a as HTMLAnchorElement).pathname);
    }),
  );
  document.querySelector("#hero-demo")?.addEventListener("click", () => {
    draft = structuredClone(sample);
    sessionStorage.setItem(DEMO_STORAGE, JSON.stringify(draft));
    nav("/demo");
  });
  document.querySelector("#reset-demo")?.addEventListener("click", () => {
    sessionStorage.removeItem(DEMO_STORAGE);
    draft = structuredClone(sample);
    saveDemo();
    render();
  });
  document.querySelector("#start-real")?.addEventListener("click", () => {
    sessionStorage.removeItem(DEMO_STORAGE);
    nav("/");
  });
  document.querySelector("#go-home")?.addEventListener("click", () => nav("/"));
  document.querySelector("#sign-out")?.addEventListener("click", async () => {
    await api<void>("/api/auth/signout", { method: "POST" });
    auth = {
      authenticated: false,
      entra_sign_in_configured: auth.entra_sign_in_configured,
      github_app_configured: auth.github_app_configured,
    };
    draft = null;
    render();
  });
  document
    .querySelector<HTMLFormElement>("#blank-form")
    ?.addEventListener("submit", saveManual);
  document
    .querySelector<HTMLFormElement>("#import-form")
    ?.addEventListener("submit", importPr);
  document
    .querySelector<HTMLFormElement>("#retention-form")
    ?.addEventListener("submit", async (event) => {
      event.preventDefault();
      const input = document.querySelector<HTMLInputElement>("#retention-days")!;
      const note = document.querySelector("#real-note")!;
      try {
        const saved = await api<{ retention_days: number }>("/api/settings", {
          method: "PUT",
          body: JSON.stringify({ retention_days: Number(input.value) }),
        });
        retentionDays = saved.retention_days;
        note.textContent = `Packets will be kept for ${retentionDays} days.`;
      } catch (error) {
        note.textContent = error instanceof Error ? error.message : "Could not save retention.";
      }
    });
  document
    .querySelectorAll<HTMLButtonElement>("[data-packet]")
    .forEach((button) =>
      button.addEventListener("click", async () => {
        try {
          const packetId = button.dataset.packet!;
          const [saved, history] = await Promise.all([
            api<any>(`/api/packets/${packetId}`),
            api<AuditEntry[]>(`/api/packets/${packetId}/audit`),
          ]);
          auditHistory = history;
          draft = {
            ...JSON.parse(saved.data),
            id: saved.id,
            title: saved.title,
            owner: saved.owner,
            status: saved.status,
            source_url: saved.source_url,
            approved_by: saved.approved_by,
            approved_at: saved.approved_at,
          };
          render();
        } catch {
          await loadAuth();
        }
      }),
    );
  document.querySelectorAll<HTMLButtonElement>("[data-resolve]").forEach((b) =>
    b.addEventListener("click", async () => {
      if (!draft) return;
      draft.checks[Number(b.dataset.resolve)].state = "done";
      try {
        if (demo) saveDemo();
        else await saveEvidence();
        render();
      } catch (error) {
        const note = document.querySelector("#approval-note")!;
        note.textContent =
          error instanceof Error
            ? error.message
            : "Could not save review evidence.";
      }
    }),
  );
  document
    .querySelector("#approve-packet")
    ?.addEventListener("click", async () => {
      if (!draft) return;
      if (demo) {
        draft.status = "approved";
        draft.approved_by = draft.owner;
        draft.approved_at = new Date().toISOString();
        saveDemo();
        render();
        return;
      }
      try {
        const saved = await api<any>(`/api/packets/${draft.id}/approve`, {
          method: "POST",
          body: JSON.stringify({
            note: "All displayed checks were reviewed in Diff Gate.",
          }),
        });
        draft = {
          ...draft,
          status: saved.status,
          approved_by: saved.approved_by,
          approved_at: saved.approved_at,
        };
        await loadAudit(saved.id);
        render();
      } catch (error) {
        const note = document.querySelector("#approval-note")!;
        note.textContent =
          error instanceof Error ? error.message : "Could not record approval.";
      }
    });
  document.querySelector("#export-packet")?.addEventListener("click", () => {
    if (!draft) return;
    const exported = demo ? draft : { ...draft, audit_history: auditHistory };
    const blob = new Blob([JSON.stringify(exported, null, 2)], {
      type: "application/json",
    });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = "diff-gate-packet.json";
    a.click();
    URL.revokeObjectURL(a.href);
  });
  document.querySelector("#delete-packet")?.addEventListener("click", async () => {
    if (!draft?.id || demo) return;
    if (!window.confirm(`Delete “${draft.title}” and its audit history?`)) return;
    try {
      await api<void>(`/api/packets/${draft.id}`, { method: "DELETE" });
      draft = null;
      auditHistory = [];
      await loadAuth();
    } catch (error) {
      const note = document.querySelector("#approval-note")!;
      note.textContent = error instanceof Error ? error.message : "Could not delete the packet.";
    }
  });
}
window.addEventListener("popstate", syncRoute);
window.addEventListener("online", () => render());
window.addEventListener("offline", () => render());
syncRoute();
