import init, {
  batch_generate,
  convert_timestamp,
  generate,
  inspect_identifier,
  resolve_local_timestamp,
  search_time_zones,
  time_zone_database_version,
} from "./pkg/uuid_factory_web.js";

const elements = {
  body: document.body,
  moduleIdentifiers: document.querySelector("#module-identifiers"),
  moduleTime: document.querySelector("#module-time"),
  identifierWorkbench: document.querySelector("#identifier-workbench"),
  timeWorkbench: document.querySelector("#time-workbench"),
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
  timeModeConvert: document.querySelector("#time-mode-convert"),
  timeModeResolve: document.querySelector("#time-mode-resolve"),
  timeConvertPanel: document.querySelector("#time-convert-panel"),
  timeResolvePanel: document.querySelector("#time-resolve-panel"),
  timeConvertForm: document.querySelector("#time-convert-form"),
  timeResolveForm: document.querySelector("#time-resolve-form"),
  timeInstantInput: document.querySelector("#time-instant-input"),
  timeZoneInput: document.querySelector("#time-zone-input"),
  timeAddZone: document.querySelector("#time-add-zone"),
  timeSuggestions: document.querySelector("#time-zone-suggestions"),
  timeSelectedZones: document.querySelector("#time-selected-zones"),
  timeConvertError: document.querySelector("#time-convert-error"),
  timeResolveError: document.querySelector("#time-resolve-error"),
  timeConvertSubmit: document.querySelector("#time-convert-submit"),
  timeResolveSubmit: document.querySelector("#time-resolve-submit"),
  timeLocalInput: document.querySelector("#time-local-input"),
  timeResolveZone: document.querySelector("#time-resolve-zone"),
  timePrimaryUtc: document.querySelector("#time-primary-utc"),
  timeConversionOutput: document.querySelector("#time-conversion-output"),
  timeConversionBody: document.querySelector("#time-conversion-output tbody"),
  timeResolutionOutput: document.querySelector("#time-resolution-output"),
  timeResultMeta: document.querySelector("#time-result-meta"),
  timeCopyOutput: document.querySelector("#time-copy-output"),
  status: document.querySelector("#status"),
};

let currentMode = "generate";
let copyPayload = "";
let timeCopyPayload = "";
let generatedCount = 0;
let selectedTimeZones = ["UTC"];
let timeZoneSuggestions = [];
let activeTimeZoneSuggestion = -1;

function setStatus(message) {
  elements.status.textContent = message;
}

function errorMessage(error) {
  if (error && typeof error.message === "string" && error.message) return error.message;
  return "Operation failed";
}

function timeErrorMessage(error) {
  if (error && typeof error.code === "string" && error.code) {
    return `${error.code}: ${errorMessage(error)}`;
  }
  return errorMessage(error);
}

function showError(element, error) {
  element.textContent = errorMessage(error);
  element.hidden = false;
}

function showTimeError(element, error) {
  element.textContent = timeErrorMessage(error);
  element.hidden = false;
}

function clearError(element) {
  element.textContent = "";
  element.hidden = true;
}

function createTimeError(code, message) {
  return { code, message };
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

function selectedTimeKind() {
  return new FormData(elements.timeConvertForm).get("time-input-kind");
}

function selectTab(activeIndex, tabs, panels) {
  tabs.forEach((tab, index) => {
    const active = index === activeIndex;
    tab.setAttribute("aria-selected", String(active));
    tab.tabIndex = active ? 0 : -1;
    panels[index].hidden = !active;
  });
}

function setModule(module) {
  const index = module === "identifiers" ? 0 : 1;
  selectTab(index, [elements.moduleIdentifiers, elements.moduleTime], [
    elements.identifierWorkbench,
    elements.timeWorkbench,
  ]);
  setStatus(module === "identifiers" ? "Identifiers module" : "Time module");
}

function setMode(mode) {
  currentMode = mode;
  const isGenerate = mode === "generate";
  selectTab(isGenerate ? 0 : 1, [elements.modeGenerate, elements.modeInspect], [
    elements.generatePanel,
    elements.inspectPanel,
  ]);
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

function setTimeMode(mode) {
  const isConvert = mode === "convert";
  selectTab(isConvert ? 0 : 1, [elements.timeModeConvert, elements.timeModeResolve], [
    elements.timeConvertPanel,
    elements.timeResolvePanel,
  ]);
  setStatus(isConvert ? "Convert instant mode" : "Resolve local mode");
}

function handleTabKeyDown(event, tabs, onSelect) {
  const keys = ["ArrowLeft", "ArrowRight", "Home", "End"];
  if (!keys.includes(event.key)) return;

  event.preventDefault();
  const currentIndex = tabs.indexOf(event.currentTarget);
  let nextIndex;

  if (event.key === "Home") nextIndex = 0;
  else if (event.key === "End") nextIndex = tabs.length - 1;
  else if (event.key === "ArrowRight") nextIndex = (currentIndex + 1) % tabs.length;
  else nextIndex = (currentIndex - 1 + tabs.length) % tabs.length;

  onSelect(nextIndex);
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

function renderSelectedTimeZones() {
  elements.timeSelectedZones.replaceChildren();
  for (const zone of selectedTimeZones) {
    const item = document.createElement("li");
    const name = document.createElement("span");
    name.textContent = zone;
    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "icon-button zone-remove";
    remove.dataset.zone = zone;
    remove.setAttribute("aria-label", `Remove ${zone}`);
    remove.title = `Remove ${zone}`;
    remove.disabled = selectedTimeZones.length === 1;
    remove.innerHTML = '<svg aria-hidden="true"><use href="#icon-x"></use></svg>';
    item.append(name, remove);
    elements.timeSelectedZones.append(item);
  }
}

function closeTimeZoneSuggestions() {
  activeTimeZoneSuggestion = -1;
  elements.timeSuggestions.hidden = true;
  elements.timeZoneInput.setAttribute("aria-expanded", "false");
  elements.timeZoneInput.removeAttribute("aria-activedescendant");
  for (const option of elements.timeSuggestions.querySelectorAll('[role="option"]')) {
    option.setAttribute("aria-selected", "false");
  }
}

function setActiveTimeZoneSuggestion(index) {
  const options = Array.from(elements.timeSuggestions.querySelectorAll('[role="option"]'));
  if (options.length === 0) {
    closeTimeZoneSuggestions();
    return;
  }

  activeTimeZoneSuggestion = Math.min(options.length - 1, Math.max(0, index));
  options.forEach((option, optionIndex) => {
    option.setAttribute("aria-selected", String(optionIndex === activeTimeZoneSuggestion));
  });
  const activeOption = options[activeTimeZoneSuggestion];
  elements.timeZoneInput.setAttribute("aria-activedescendant", activeOption.id);
  activeOption.scrollIntoView({ block: "nearest" });
}

function renderTimeZoneSuggestions() {
  const query = elements.timeZoneInput.value.trim();
  timeZoneSuggestions = [];
  activeTimeZoneSuggestion = -1;
  elements.timeSuggestions.replaceChildren();
  if (!query) {
    closeTimeZoneSuggestions();
    return;
  }

  try {
    timeZoneSuggestions = Array.from(search_time_zones(query)).slice(0, 6);
    for (const [index, zone] of timeZoneSuggestions.entries()) {
      const option = document.createElement("li");
      option.id = `time-zone-suggestion-${index}`;
      option.role = "option";
      option.dataset.zone = zone;
      option.setAttribute("aria-selected", "false");
      option.textContent = zone;
      elements.timeSuggestions.append(option);
    }
    const expanded = timeZoneSuggestions.length > 0;
    elements.timeSuggestions.hidden = !expanded;
    elements.timeZoneInput.setAttribute("aria-expanded", String(expanded));
    elements.timeZoneInput.removeAttribute("aria-activedescendant");
  } catch (error) {
    closeTimeZoneSuggestions();
    showTimeError(elements.timeConvertError, error);
  }
}

function handleTimeZoneInputKeyDown(event) {
  if (event.key === "Escape") {
    if (!elements.timeSuggestions.hidden) {
      event.preventDefault();
      closeTimeZoneSuggestions();
    }
    return;
  }

  if (event.key === "Enter") {
    if (!elements.timeSuggestions.hidden && activeTimeZoneSuggestion >= 0) {
      event.preventDefault();
      addTimeZone(timeZoneSuggestions[activeTimeZoneSuggestion]);
    }
    return;
  }

  if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
  event.preventDefault();
  if (elements.timeSuggestions.hidden) renderTimeZoneSuggestions();
  if (timeZoneSuggestions.length === 0) return;

  if (activeTimeZoneSuggestion < 0) {
    setActiveTimeZoneSuggestion(event.key === "ArrowDown" ? 0 : timeZoneSuggestions.length - 1);
  } else {
    const delta = event.key === "ArrowDown" ? 1 : -1;
    setActiveTimeZoneSuggestion(activeTimeZoneSuggestion + delta);
  }
}

function addTimeZone(value = elements.timeZoneInput.value) {
  clearError(elements.timeConvertError);
  const requestedZone = value.trim();

  try {
    if (selectedTimeZones.length >= 8) {
      throw createTimeError("TIMEZONE_LIMIT_EXCEEDED", "Time zone count must be between 1 and 8");
    }
    const zone = Array.from(search_time_zones(requestedZone))
      .find((candidate) => candidate.toLowerCase() === requestedZone.toLowerCase());
    if (!zone) throw createTimeError("INVALID_TIMEZONE", "Invalid IANA time zone");
    if (selectedTimeZones.includes(zone)) {
      throw createTimeError("DUPLICATE_TIMEZONE", "Duplicate IANA time zone");
    }
    selectedTimeZones = [...selectedTimeZones, zone];
    elements.timeZoneInput.value = "";
    closeTimeZoneSuggestions();
    renderSelectedTimeZones();
    setStatus(`Added ${zone}`);
  } catch (error) {
    showTimeError(elements.timeConvertError, error);
    setStatus("Time zone selection failed");
  }
}

function removeTimeZone(zone) {
  if (selectedTimeZones.length === 1) {
    showTimeError(
      elements.timeConvertError,
      createTimeError("TIMEZONE_LIMIT_EXCEEDED", "Time zone count must be between 1 and 8")
    );
    return;
  }
  selectedTimeZones = selectedTimeZones.filter((selectedZone) => selectedZone !== zone);
  renderSelectedTimeZones();
  setStatus(`Removed ${zone}`);
}

function setTimePrimaryResult(instant) {
  elements.timePrimaryUtc.querySelector('[data-time-field="utc"]').textContent = instant.utc_rfc3339;
  elements.timePrimaryUtc.querySelector('[data-time-field="seconds"]').textContent = instant.unix_seconds;
  elements.timePrimaryUtc.querySelector('[data-time-field="milliseconds"]').textContent = instant.unix_milliseconds;
  elements.timePrimaryUtc.hidden = false;
}

function appendCell(row, label, value) {
  const cell = document.createElement("td");
  cell.dataset.label = label;
  cell.textContent = value;
  row.append(cell);
}

function renderConversion(result) {
  setTimePrimaryResult(result.instant);
  elements.timeConversionBody.replaceChildren();
  for (const zone of result.zones) {
    const row = document.createElement("tr");
    row.dataset.zone = zone.zone;
    appendCell(row, "Zone", zone.zone);
    appendCell(row, "Local time", zone.local_datetime);
    appendCell(row, "Offset", zone.offset);
    appendCell(row, "Abbreviation", zone.abbreviation);
    appendCell(row, "DST", zone.is_dst === null ? "Unknown" : zone.is_dst ? "Yes" : "No");
    elements.timeConversionBody.append(row);
  }
  elements.timeConversionOutput.hidden = false;
  elements.timeResolutionOutput.hidden = true;
  elements.timeResultMeta.textContent = `IANA tzdb ${result.tzdb_version} - ${result.zones.length} zones`;
  timeCopyPayload = JSON.stringify(result, null, 2);
  elements.timeCopyOutput.disabled = false;
}

function appendResolutionField(container, label, value) {
  const item = document.createElement("div");
  const term = document.createElement("dt");
  const description = document.createElement("dd");
  term.textContent = label;
  description.textContent = value;
  item.append(term, description);
  container.append(item);
}

function appendCandidate(container, label, candidate) {
  const candidateOutput = document.createElement("dl");
  candidateOutput.className = "resolution-candidate";
  appendResolutionField(candidateOutput, label, candidate.utc_rfc3339);
  appendResolutionField(candidateOutput, "Unix seconds", candidate.unix_seconds);
  appendResolutionField(candidateOutput, "Offset", candidate.offset);
  appendResolutionField(candidateOutput, "Abbreviation", candidate.abbreviation);
  container.append(candidateOutput);
}

function renderResolution(result) {
  const output = elements.timeResolutionOutput;
  output.replaceChildren();
  const status = document.createElement("p");
  status.id = "time-resolution-status";
  status.className = "resolution-status";
  const resolution = result.resolution;

  if (resolution.status === "UNAMBIGUOUS") {
    status.textContent = "Unambiguous";
    appendCandidate(output, "Instant", resolution.instant);
  } else if (resolution.status === "GAP") {
    status.textContent = "Gap";
    const gap = document.createElement("dl");
    gap.className = "resolution-candidate";
    appendResolutionField(gap, "Before offset", resolution.before_offset);
    appendResolutionField(gap, "After offset", resolution.after_offset);
    output.append(gap);
  } else {
    status.textContent = "Fold";
    appendCandidate(output, "Earlier", resolution.earlier);
    appendCandidate(output, "Later", resolution.later);
  }

  output.prepend(status);
  output.hidden = false;
  elements.timePrimaryUtc.hidden = true;
  elements.timeConversionOutput.hidden = true;
  elements.timeResultMeta.textContent = `IANA tzdb ${result.tzdb_version} - ${result.zone}`;
  timeCopyPayload = JSON.stringify(result, null, 2);
  elements.timeCopyOutput.disabled = false;
}

async function handleTimeConvert(event) {
  event.preventDefault();
  clearError(elements.timeConvertError);

  try {
    const result = convert_timestamp(
      selectedTimeKind(),
      elements.timeInstantInput.value,
      selectedTimeZones
    );
    renderConversion(result);
    setStatus(`Converted ${result.zones.length} time zones`);
  } catch (error) {
    showTimeError(elements.timeConvertError, error);
    setStatus("Time conversion failed");
  }
}

async function handleTimeResolve(event) {
  event.preventDefault();
  clearError(elements.timeResolveError);

  try {
    const result = resolve_local_timestamp(
      elements.timeLocalInput.value,
      elements.timeResolveZone.value
    );
    renderResolution(result);
    setStatus("Local time resolved");
  } catch (error) {
    showTimeError(elements.timeResolveError, error);
    setStatus("Local time resolution failed");
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

async function copyTimeOutput() {
  if (!timeCopyPayload) return;

  try {
    await navigator.clipboard.writeText(timeCopyPayload);
    elements.timeCopyOutput.setAttribute("aria-label", "Copied");
    elements.timeCopyOutput.title = "Copied";
    setStatus("Copied time result");
    window.setTimeout(() => {
      elements.timeCopyOutput.setAttribute("aria-label", "Copy time result");
      elements.timeCopyOutput.title = "Copy time result";
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
  const moduleTabs = [elements.moduleIdentifiers, elements.moduleTime];
  const identifierTabs = [elements.modeGenerate, elements.modeInspect];
  const timeTabs = [elements.timeModeConvert, elements.timeModeResolve];

  elements.moduleIdentifiers.addEventListener("click", () => setModule("identifiers"));
  elements.moduleTime.addEventListener("click", () => setModule("time"));
  elements.moduleIdentifiers.addEventListener("keydown", (event) => {
    handleTabKeyDown(event, moduleTabs, (index) => setModule(index === 0 ? "identifiers" : "time"));
  });
  elements.moduleTime.addEventListener("keydown", (event) => {
    handleTabKeyDown(event, moduleTabs, (index) => setModule(index === 0 ? "identifiers" : "time"));
  });
  elements.modeGenerate.addEventListener("click", () => setMode("generate"));
  elements.modeInspect.addEventListener("click", () => setMode("inspect"));
  elements.modeGenerate.addEventListener("keydown", (event) => {
    handleTabKeyDown(event, identifierTabs, (index) => setMode(index === 0 ? "generate" : "inspect"));
  });
  elements.modeInspect.addEventListener("keydown", (event) => {
    handleTabKeyDown(event, identifierTabs, (index) => setMode(index === 0 ? "generate" : "inspect"));
  });
  elements.timeModeConvert.addEventListener("click", () => setTimeMode("convert"));
  elements.timeModeResolve.addEventListener("click", () => setTimeMode("resolve"));
  elements.timeModeConvert.addEventListener("keydown", (event) => {
    handleTabKeyDown(event, timeTabs, (index) => setTimeMode(index === 0 ? "convert" : "resolve"));
  });
  elements.timeModeResolve.addEventListener("keydown", (event) => {
    handleTabKeyDown(event, timeTabs, (index) => setTimeMode(index === 0 ? "convert" : "resolve"));
  });
  elements.generateForm.addEventListener("submit", handleGenerate);
  elements.inspectForm.addEventListener("submit", handleInspect);
  elements.timeConvertForm.addEventListener("submit", handleTimeConvert);
  elements.timeResolveForm.addEventListener("submit", handleTimeResolve);
  elements.countDecrease.addEventListener("click", () => stepCount(-1));
  elements.countIncrease.addEventListener("click", () => stepCount(1));
  elements.copyOutput.addEventListener("click", copyOutput);
  elements.downloadOutput.addEventListener("click", downloadOutput);
  elements.timeCopyOutput.addEventListener("click", copyTimeOutput);
  elements.timeAddZone.addEventListener("click", () => addTimeZone());
  elements.timeZoneInput.addEventListener("input", renderTimeZoneSuggestions);
  elements.timeZoneInput.addEventListener("keydown", handleTimeZoneInputKeyDown);
  elements.timeSuggestions.addEventListener("click", (event) => {
    const option = event.target.closest('[role="option"][data-zone]');
    if (option) addTimeZone(option.dataset.zone);
  });
  elements.timeSelectedZones.addEventListener("click", (event) => {
    const button = event.target.closest("button[data-zone]");
    if (button) removeTimeZone(button.dataset.zone);
  });
}

async function start() {
  bindEvents();
  renderSelectedTimeZones();

  try {
    await init();
    elements.generateSubmit.disabled = false;
    elements.inspectSubmit.disabled = false;
    elements.timeAddZone.disabled = false;
    elements.timeConvertSubmit.disabled = false;
    elements.timeResolveSubmit.disabled = false;
    elements.timeResultMeta.textContent = `IANA tzdb ${time_zone_database_version()}`;
    elements.body.dataset.ready = "true";
    setStatus("Ready");
  } catch (error) {
    console.error(error);
    showError(elements.generateError, new Error("Workbench failed to load"));
    setStatus("Load failed");
  }
}

start();
