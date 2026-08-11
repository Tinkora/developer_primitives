import init, { batch_generate, generate, inspect_identifier } from "./pkg/uuid_factory_web.js";

const elements = {
  body: document.body,
  modeGenerate: document.querySelector("#mode-generate"),
  modeInspect: document.querySelector("#mode-inspect"),
  generatePanel: document.querySelector("#generate-panel"),
  inspectPanel: document.querySelector("#inspect-panel"),
  generateForm: document.querySelector("#generate-form"),
  inspectForm: document.querySelector("#inspect-form"),
  generateSubmit: document.querySelector("#generate-submit"),
  inspectSubmit: document.querySelector("#inspect-submit"),
  count: document.querySelector("#count"),
  countDecrease: document.querySelector("#count-decrease"),
  countIncrease: document.querySelector("#count-increase"),
  identifierInput: document.querySelector("#identifier-input"),
  generateError: document.querySelector("#generate-error"),
  inspectError: document.querySelector("#inspect-error"),
  generateOutput: document.querySelector("#generate-output"),
  inspectOutput: document.querySelector("#inspect-output"),
  resultCount: document.querySelector("#result-count"),
  copyOutput: document.querySelector("#copy-output"),
  downloadOutput: document.querySelector("#download-output"),
  status: document.querySelector("#status"),
};

let currentMode = "generate";
let copyPayload = "";
let generatedCount = 0;

function setStatus(message) {
  elements.status.textContent = message;
}

function errorMessage(error) {
  if (error && typeof error.message === "string" && error.message) return error.message;
  return "Operation failed";
}

function showError(element, error) {
  element.textContent = errorMessage(error);
  element.hidden = false;
}

function clearError(element) {
  element.textContent = "";
  element.hidden = true;
}

function normalizedCount() {
  const count = Number(elements.count.value);
  if (!Number.isInteger(count) || count < 1 || count > 10_000) {
    throw new Error("Count must be between 1 and 10000");
  }
  return count;
}

function selectedKind() {
  return new FormData(elements.generateForm).get("kind");
}

function setMode(mode) {
  currentMode = mode;
  const isGenerate = mode === "generate";
  elements.modeGenerate.setAttribute("aria-selected", String(isGenerate));
  elements.modeInspect.setAttribute("aria-selected", String(!isGenerate));
  elements.modeGenerate.tabIndex = isGenerate ? 0 : -1;
  elements.modeInspect.tabIndex = isGenerate ? -1 : 0;
  elements.generatePanel.hidden = !isGenerate;
  elements.inspectPanel.hidden = isGenerate;
  elements.generateOutput.hidden = !isGenerate;
  elements.inspectOutput.hidden = isGenerate;
  elements.downloadOutput.hidden = !isGenerate;

  if (isGenerate) {
    copyPayload = elements.generateOutput.value;
    generatedCount = copyPayload ? copyPayload.split("\n").length : 0;
    elements.resultCount.textContent = `${generatedCount} ${generatedCount === 1 ? "identifier" : "identifiers"}`;
    elements.copyOutput.disabled = !copyPayload;
    elements.downloadOutput.disabled = !copyPayload;
    setStatus("Generate mode");
  } else {
    copyPayload = elements.inspectOutput.dataset.canonical || "";
    elements.resultCount.textContent = "Inspection";
    elements.copyOutput.disabled = !copyPayload;
    setStatus("Inspect mode");
  }
}

function handleModeKeyDown(event) {
  const keys = ["ArrowLeft", "ArrowRight", "Home", "End"];
  if (!keys.includes(event.key)) return;

  event.preventDefault();
  const tabs = [elements.modeGenerate, elements.modeInspect];
  const currentIndex = tabs.indexOf(event.currentTarget);
  let nextIndex;

  if (event.key === "Home") nextIndex = 0;
  else if (event.key === "End") nextIndex = tabs.length - 1;
  else if (event.key === "ArrowRight") nextIndex = (currentIndex + 1) % tabs.length;
  else nextIndex = (currentIndex - 1 + tabs.length) % tabs.length;

  const nextMode = nextIndex === 0 ? "generate" : "inspect";
  setMode(nextMode);
  tabs[nextIndex].focus();
}

function updateInspection(inspection) {
  const fields = ["kind", "canonical", "version", "variant", "timestamp_ms"];
  for (const field of fields) {
    const target = elements.inspectOutput.querySelector(`[data-field="${field}"]`);
    target.textContent = inspection[field] ?? "-";
  }
  elements.inspectOutput.dataset.canonical = inspection.canonical;
  copyPayload = inspection.canonical;
  elements.copyOutput.disabled = false;
}

function resetInspection() {
  for (const target of elements.inspectOutput.querySelectorAll("dd")) {
    target.textContent = "-";
  }
  elements.inspectOutput.dataset.canonical = "";
  copyPayload = "";
  elements.copyOutput.disabled = true;
}

async function handleGenerate(event) {
  event.preventDefault();
  clearError(elements.generateError);

  try {
    const count = normalizedCount();
    const kind = selectedKind();
    const identifiers = count === 1 ? [generate(kind)] : Array.from(batch_generate(kind, count));
    copyPayload = identifiers.join("\n");
    generatedCount = identifiers.length;
    elements.generateOutput.value = copyPayload;
    elements.resultCount.textContent = `${generatedCount} ${generatedCount === 1 ? "identifier" : "identifiers"}`;
    elements.copyOutput.disabled = false;
    elements.downloadOutput.disabled = false;
    setStatus(`Generated ${generatedCount} ${generatedCount === 1 ? "identifier" : "identifiers"}`);
  } catch (error) {
    showError(elements.generateError, error);
    setStatus("Generation failed");
  }
}

async function handleInspect(event) {
  event.preventDefault();
  clearError(elements.inspectError);
  resetInspection();

  try {
    const inspection = inspect_identifier(elements.identifierInput.value);
    updateInspection(inspection);
    setStatus("Inspection complete");
  } catch (error) {
    showError(elements.inspectError, error);
    setStatus("Inspection failed");
  }
}

async function copyOutput() {
  if (!copyPayload) return;

  try {
    await navigator.clipboard.writeText(copyPayload);
    const message = currentMode === "generate"
      ? `Copied ${generatedCount} ${generatedCount === 1 ? "identifier" : "identifiers"}`
      : "Copied canonical identifier";
    elements.copyOutput.setAttribute("aria-label", "Copied");
    elements.copyOutput.title = "Copied";
    setStatus(message);
    window.setTimeout(() => {
      elements.copyOutput.setAttribute("aria-label", "Copy output");
      elements.copyOutput.title = "Copy output";
    }, 1500);
  } catch {
    setStatus("Copy failed");
  }
}

function downloadOutput() {
  if (!copyPayload || currentMode !== "generate") return;

  const blob = new Blob([`${copyPayload}\n`], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = "tinkora-identifiers.txt";
  document.body.append(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
  setStatus(`Downloaded ${generatedCount} ${generatedCount === 1 ? "identifier" : "identifiers"}`);
}

function stepCount(delta) {
  const current = Number.parseInt(elements.count.value, 10) || 1;
  elements.count.value = String(Math.min(10_000, Math.max(1, current + delta)));
}

function bindEvents() {
  elements.modeGenerate.addEventListener("click", () => setMode("generate"));
  elements.modeInspect.addEventListener("click", () => setMode("inspect"));
  elements.modeGenerate.addEventListener("keydown", handleModeKeyDown);
  elements.modeInspect.addEventListener("keydown", handleModeKeyDown);
  elements.generateForm.addEventListener("submit", handleGenerate);
  elements.inspectForm.addEventListener("submit", handleInspect);
  elements.countDecrease.addEventListener("click", () => stepCount(-1));
  elements.countIncrease.addEventListener("click", () => stepCount(1));
  elements.copyOutput.addEventListener("click", copyOutput);
  elements.downloadOutput.addEventListener("click", downloadOutput);
}

async function start() {
  bindEvents();

  try {
    await init();
    elements.generateSubmit.disabled = false;
    elements.inspectSubmit.disabled = false;
    elements.body.dataset.ready = "true";
    setStatus("Ready");
  } catch (error) {
    console.error(error);
    showError(elements.generateError, new Error("Workbench failed to load"));
    setStatus("Load failed");
  }
}

start();
