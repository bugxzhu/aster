// Tauri comparison app. Same functional scope as the Aster file manager (requirements.md).
import { files, locales } from "./data.js";

const { invoke } = window.__TAURI__.core;
const LOCALE_ORDER = ["en", "zh-CN"];
const state = { query: "", selected: null, dark: false, locale: 0 };

const $ = (id) => document.getElementById(id);
const t = (key) => locales[LOCALE_ORDER[state.locale]][key];

function visibleRows() {
  const q = state.query.toLowerCase();
  return files.filter((f) => f.name.toLowerCase().includes(q));
}

function renderList() {
  const list = $("file-list");
  list.replaceChildren(...visibleRows().map((f) => {
    const li = document.createElement("li");
    li.className = "file-row" + (state.selected === f.name ? " selected" : "");
    li.innerHTML = `<span class="name"></span><span class="kind"></span><span class="modified"></span>`;
    li.querySelector(".name").textContent = f.name;
    li.querySelector(".kind").textContent = f.kind;
    li.querySelector(".modified").textContent = f.modified;
    li.addEventListener("click", () => { state.selected = f.name; render(); });
    return li;
  }));
}

function renderLabels() {
  document.querySelectorAll("[data-i18n]").forEach((el) => { el.textContent = t(el.dataset.i18n); });
  $("search").placeholder = t("search_placeholder");
  $("btn-theme").textContent = state.dark ? t("theme_light") : t("theme_dark");
  document.documentElement.lang = LOCALE_ORDER[state.locale];
}

function renderDetail() {
  $("detail-text").textContent = state.selected ?? t("no_selection");
}

function render() {
  renderLabels();
  renderList();
  renderDetail();
  document.body.classList.toggle("dark", state.dark);
}

$("search").addEventListener("input", (e) => { state.query = e.target.value; renderList(); });
$("btn-theme").addEventListener("click", () => { state.dark = !state.dark; render(); });
$("btn-language").addEventListener("click", () => { state.locale = (state.locale + 1) % LOCALE_ORDER.length; render(); });
// New folder and Refresh have no business action in this comparison (same as the Aster spike).

render();

// Timing and geometry hooks. Only active when the Rust side injects the flags (see src-tauri).
function collectGeometry() {
  const ids = ["sidebar", "content", "toolbar", "search", "file-list", "detail"];
  const pick = (el) => { const r = el.getBoundingClientRect(); return { x: r.x, y: r.y, width: r.width, height: r.height }; };
  const out = {};
  for (const id of ids) {
    const el = id === "sidebar" ? document.querySelector(".sidebar")
      : id === "content" ? document.querySelector(".content")
      : id === "toolbar" ? document.querySelector(".toolbar")
      : id === "search" ? $("search") : id === "file-list" ? $("file-list") : document.querySelector(".detail");
    out[id] = pick(el);
  }
  out["all_files_label"] = pick(document.querySelector(".nav-item"));
  out["recent_label"] = pick(document.querySelectorAll(".nav-item")[1]);
  out["window"] = { width: window.innerWidth, height: window.innerHeight };
  return out;
}

if (window.__ASTER_COMPARE__) {
  requestAnimationFrame(() => requestAnimationFrame(() => {
    invoke("report_geometry", { json: JSON.stringify(collectGeometry()) });
  }));
}
if (window.__ASTER_TIMING__) {
  requestAnimationFrame(() => requestAnimationFrame(() => invoke("report_first_frame")));
}

// Self-test (only with ASTER_COMPARE_SELFTEST): drives the real page through its controls and
// reports the outcome of each functional requirement that can be checked without a human.
async function runSelfTest() {
  const results = {};
  const type = (text) => { const s = $("search"); s.value = text; s.dispatchEvent(new Event("input")); };
  const rowCount = () => document.querySelectorAll("#file-list .file-row").length;
  const nameOf = (text) => [...document.querySelectorAll("#file-list .name")].find((n) => n.textContent === text);
  type("zzz-no-such-file");
  results["F6_empty_result_rows"] = rowCount();
  type("report");
  results["F4_filter_matches"] = rowCount();
  type("");
  results["F4_filter_restored"] = rowCount();
  nameOf("todo.txt").closest(".file-row").click();
  results["F7_detail_after_select"] = $("detail-text").textContent;
  const before = document.body.classList.contains("dark");
  $("btn-theme").click();
  results["F9_theme_toggled"] = document.body.classList.contains("dark") !== before;
  results["F9_theme_label"] = $("btn-theme").textContent;
  const enLabel = $("btn-new-folder").textContent;
  $("btn-language").click();
  results["F10_zh_new_folder"] = $("btn-new-folder").textContent;
  $("btn-language").click();
  results["F10_restored"] = $("btn-new-folder").textContent === enLabel;
  const long = nameOf("Long file name that must be truncated without breaking layout.txt");
  results["F11_long_name_elided"] = long.scrollWidth > long.clientWidth;
  return results;
}
if (window.__ASTER_SELFTEST__) {
  requestAnimationFrame(() => requestAnimationFrame(async () => {
    let payload;
    try {
      payload = await runSelfTest();
    } catch (e) {
      payload = { error: String(e && e.stack ? e.stack : e) };
    }
    invoke("report_selftest", { json: JSON.stringify(payload) });
  }));
}
