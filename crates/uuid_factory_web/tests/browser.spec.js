import { expect, test } from "@playwright/test";

async function openWorkbench(page) {
  await page.goto("/");
  await expect(page.locator("body")).toHaveAttribute("data-ready", "true");
}

async function openTimeWorkbench(page) {
  await openWorkbench(page);
  await page.locator("#module-time").click();
  await expect(page.locator("#time-workbench")).toBeVisible();
}

async function addTimeZone(page, zone) {
  await page.locator("#time-zone-input").fill(zone);
  await page.locator("#time-add-zone").click();
  await expect(page.locator("#time-selected-zones")).toContainText(zone);
}

function contrastRatio(first, second) {
  const luminance = (color) => {
    const channels = color.match(/[\d.]+/g).slice(0, 3).map((value) => {
      const normalized = Number(value) / 255;
      return normalized <= 0.04045
        ? normalized / 12.92
        : ((normalized + 0.055) / 1.055) ** 2.4;
    });
    return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
  };

  const lighter = Math.max(luminance(first), luminance(second));
  const darker = Math.min(luminance(first), luminance(second));
  return (lighter + 0.05) / (darker + 0.05);
}

test("generate mode exposes stable controls and ordered output", async ({ page }) => {
  await openWorkbench(page);

  await expect(page.locator("#mode-generate")).toHaveAttribute("aria-selected", "true");
  await expect(page.locator("#generate-panel")).toBeVisible();
  await page.locator('label[for="kind-uuid-v7"]').click();
  await expect(page.locator("#kind-uuid-v7")).toBeChecked();
  await page.locator("#count").fill("3");
  await page.locator("#generate-submit").click();

  const lines = (await page.locator("#generate-output").inputValue()).trim().split("\n");
  expect(lines).toHaveLength(3);
  expect(lines.every((identifier) => identifier[14] === "7")).toBeTruthy();
  await expect(page.locator("#result-count")).toHaveText("3 identifiers");
});

test("enter submits generation and copy reports feedback", async ({ page, context }) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await openWorkbench(page);

  await page.locator("#count").fill("2");
  await page.locator("#count").press("Enter");
  await expect(page.locator("#generate-output")).not.toHaveValue("");
  await page.locator("#copy-output").click();
  await expect(page.locator("#status")).toHaveText("Copied 2 identifiers");
  await expect(page.locator("#copy-output")).toHaveAttribute("aria-label", "Copied");
});

test("generated output downloads as a text file", async ({ page }) => {
  await openWorkbench(page);

  await page.locator("#generate-submit").click();
  const downloadPromise = page.waitForEvent("download");
  await page.locator("#download-output").click();
  const download = await downloadPromise;
  expect(download.suggestedFilename()).toBe("tinkora-identifiers.txt");
  await expect(page.locator("#status")).toHaveText("Downloaded 1 identifier");
});

test("generation rejects fractional counts", async ({ page }) => {
  await openWorkbench(page);

  await page.locator("#count").fill("1.5");
  await page.locator("#generate-submit").click();
  await expect(page.locator("#generate-error")).toHaveText("Count must be between 1 and 10000");
  await expect(page.locator("#generate-output")).toHaveValue("");
});

test("inspect mode returns structured metadata and rejects invalid input", async ({ page }) => {
  await openWorkbench(page);

  await page.locator("#mode-inspect").click();
  await expect(page.locator("#mode-inspect")).toHaveAttribute("aria-selected", "true");
  await expect(page.locator("#inspect-panel")).toBeVisible();
  await page
    .locator("#identifier-input")
    .fill("550e8400-e29b-41d4-a716-446655440000");
  await page.locator("#inspect-submit").click();
  await expect(page.locator("[data-field=kind]")).toHaveText("uuid");
  await expect(page.locator("[data-field=version]")).toHaveText("4");
  await expect(page.locator("[data-field=variant]")).toHaveText("RFC4122");

  await page.locator("#identifier-input").fill("not-an-id");
  await page.locator("#inspect-submit").click();
  await expect(page.locator("#inspect-error")).toContainText("Invalid identifier");
});

test("page makes no external requests and logs no runtime errors", async ({ page }) => {
  const externalRequests = [];
  const consoleErrors = [];
  const pageErrors = [];
  page.on("request", (request) => {
    const url = new URL(request.url());
    if (url.origin !== "http://127.0.0.1:4173") externalRequests.push(request.url());
  });
  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(message.text());
  });
  page.on("pageerror", (error) => pageErrors.push(error.message));

  await openWorkbench(page);
  await page.locator("#module-time").click();
  await page.locator("#time-instant-input").fill("0");
  await page.locator("#time-convert-submit").click();
  await expect(page.locator("#time-conversion-output tbody tr")).toHaveCount(1);

  expect(externalRequests).toEqual([]);
  expect(consoleErrors).toEqual([]);
  expect(pageErrors).toEqual([]);
});

test("keyboard focus is visible and status updates are announced", async ({ page }) => {
  await openWorkbench(page);

  await expect(page.locator("#status")).toHaveAttribute("aria-live", "polite");
  await page.keyboard.press("Tab");
  await expect(page.locator("#module-identifiers")).toBeFocused();
  const focusStyle = await page.locator("#module-identifiers").evaluate((element) => {
    const style = getComputedStyle(element);
    return { style: style.outlineStyle, width: style.outlineWidth };
  });
  expect(focusStyle.style).not.toBe("none");
  expect(focusStyle.width).not.toBe("0px");
});

test("mode tabs implement arrow-key focus and selection", async ({ page }) => {
  await openWorkbench(page);

  await page.locator("#mode-generate").focus();
  await page.keyboard.press("ArrowRight");
  await expect(page.locator("#mode-inspect")).toBeFocused();
  await expect(page.locator("#mode-inspect")).toHaveAttribute("aria-selected", "true");
  await expect(page.locator("#mode-generate")).toHaveAttribute("tabindex", "-1");

  await page.keyboard.press("Home");
  await expect(page.locator("#mode-generate")).toBeFocused();
  await expect(page.locator("#mode-generate")).toHaveAttribute("aria-selected", "true");
  await expect(page.locator("#mode-inspect")).toHaveAttribute("tabindex", "-1");
});

test("inspect placeholder meets normal-text contrast", async ({ page }) => {
  await openWorkbench(page);
  await page.locator("#mode-inspect").click();

  const colors = await page.locator("#identifier-input").evaluate((element) => ({
    background: getComputedStyle(element).backgroundColor,
    placeholder: getComputedStyle(element, "::placeholder").color,
  }));
  expect(contrastRatio(colors.placeholder, colors.background)).toBeGreaterThanOrEqual(4.5);
});

test("layout has no horizontal viewport overflow", async ({ page }) => {
  await openWorkbench(page);

  await expect(page.locator(".brand")).toContainText("Tinkora");
  await expect(page.locator(".brand")).toBeVisible();
  const dimensions = await page.evaluate(() => ({
    clientWidth: document.documentElement.clientWidth,
    scrollWidth: document.documentElement.scrollWidth,
  }));
  expect(dimensions.scrollWidth).toBeLessThanOrEqual(dimensions.clientWidth);
});

test("time workbench switches modules without losing the product header", async ({ page }) => {
  await openTimeWorkbench(page);

  await expect(page.locator(".brand")).toContainText("Tinkora");
  await expect(page.locator(".product-title h1")).toHaveText("Developer Primitives");
  await expect(page.locator("#module-time")).toHaveAttribute("aria-selected", "true");
  await expect(page.locator("#module-identifiers")).toHaveAttribute("aria-controls", "identifier-workbench");
  await expect(page.locator("#identifier-workbench")).toHaveAttribute("role", "tabpanel");
  await expect(page.locator("#identifier-workbench")).toHaveAttribute("aria-labelledby", "module-identifiers");
  await expect(page.locator("#module-time")).toHaveAttribute("aria-controls", "time-workbench");
  await expect(page.locator("#time-workbench")).toHaveAttribute("role", "tabpanel");
  await expect(page.locator("#time-workbench")).toHaveAttribute("aria-labelledby", "module-time");
  await expect(page.locator("#time-convert-panel")).toBeVisible();
});

test("time workbench converts an instant in selected zone order", async ({ page }) => {
  await openTimeWorkbench(page);

  await addTimeZone(page, "America/New_York");
  await addTimeZone(page, "Asia/Shanghai");
  await page.locator("#time-instant-input").fill("0");
  await page.locator("#time-convert-submit").click();

  const rows = page.locator("#time-conversion-output tbody tr");
  await expect(rows).toHaveCount(3);
  await expect(rows.nth(0)).toHaveAttribute("data-zone", "UTC");
  await expect(rows.nth(1)).toHaveAttribute("data-zone", "America/New_York");
  await expect(rows.nth(2)).toHaveAttribute("data-zone", "Asia/Shanghai");
  await expect(page.locator("#time-primary-utc")).toContainText("1970-01-01T00:00:00Z");
});

test("time workbench reports an invalid IANA zone inline with its stable code", async ({ page }) => {
  await openTimeWorkbench(page);

  await page.locator("#time-zone-input").fill("Mars/Olympus");
  await page.locator("#time-add-zone").click();

  await expect(page.locator("#time-convert-error")).toHaveText(
    "INVALID_TIMEZONE: Invalid IANA time zone"
  );
  await expect(page.locator("#time-selected-zones")).not.toContainText("Mars/Olympus");
});

test("time workbench displays a DST gap without inventing an instant", async ({ page }) => {
  await openTimeWorkbench(page);
  await page.locator("#time-mode-resolve").click();
  await page.locator("#time-local-input").fill("2026-03-08T02:30:00");
  await page.locator("#time-resolve-zone").fill("America/New_York");
  await page.locator("#time-resolve-submit").click();

  await expect(page.locator("#time-resolution-status")).toHaveText("Gap");
  await expect(page.locator("#time-resolution-output")).toContainText("-05:00");
  await expect(page.locator("#time-resolution-output")).toContainText("-04:00");
  await expect(page.locator("#time-resolution-output")).not.toContainText("unix_seconds");
});

test("time workbench displays both candidates for a DST fold", async ({ page }) => {
  await openTimeWorkbench(page);
  await page.locator("#time-mode-resolve").click();
  await page.locator("#time-local-input").fill("2026-11-01T01:30:00");
  await page.locator("#time-resolve-zone").fill("America/New_York");
  await page.locator("#time-resolve-submit").click();

  await expect(page.locator("#time-resolution-status")).toHaveText("Fold");
  await expect(page.locator("#time-resolution-output")).toContainText("Earlier");
  await expect(page.locator("#time-resolution-output")).toContainText("Later");
  await expect(page.locator("#time-resolution-output")).toContainText("1793511000");
  await expect(page.locator("#time-resolution-output")).toContainText("1793514600");
});

test("time workbench keeps exactly eight selected zones without viewport overflow", async ({ page }) => {
  await openTimeWorkbench(page);
  const submitBefore = await page.locator("#time-convert-submit").boundingBox();
  const zones = [
    "America/New_York",
    "Asia/Shanghai",
    "Europe/London",
    "Asia/Kolkata",
    "Australia/Sydney",
    "Europe/Paris",
    "America/Los_Angeles",
  ];

  for (const zone of zones) await addTimeZone(page, zone);
  await expect(page.locator("#time-selected-zones li")).toHaveCount(8);
  const submitAfter = await page.locator("#time-convert-submit").boundingBox();
  expect(submitBefore).not.toBeNull();
  expect(submitAfter).not.toBeNull();
  expect(Math.abs(submitAfter.y - submitBefore.y)).toBeLessThanOrEqual(1);
  await page.locator("#time-zone-input").fill("Europe/Berlin");
  await page.locator("#time-add-zone").click();
  await expect(page.locator("#time-convert-error")).toContainText("TIMEZONE_LIMIT_EXCEEDED");

  const dimensions = await page.evaluate(() => ({
    clientWidth: document.documentElement.clientWidth,
    scrollWidth: document.documentElement.scrollWidth,
  }));
  expect(dimensions.scrollWidth).toBeLessThanOrEqual(dimensions.clientWidth);
});

test("time zone combobox supports keyboard selection and dismissal", async ({ page }) => {
  await openTimeWorkbench(page);
  const input = page.getByRole("combobox", { name: "Add IANA time zone" });
  const listbox = page.getByRole("listbox", { name: "Time zone suggestions" });
  const newYork = page.getByRole("option", { name: "America/New_York" });

  await input.fill("America/New_York");
  await expect(input).toHaveAttribute("aria-expanded", "true");
  await expect(listbox).toBeVisible();
  await page.keyboard.press("ArrowDown");
  await expect(newYork).toHaveAttribute("aria-selected", "true");
  await expect(input).toHaveAttribute("aria-activedescendant", await newYork.getAttribute("id"));
  await page.keyboard.press("Enter");
  await expect(page.locator("#time-selected-zones")).toContainText("America/New_York");
  await expect(input).toHaveValue("");
  await expect(input).toHaveAttribute("aria-expanded", "false");

  await input.fill("Europe");
  await expect(input).toHaveAttribute("aria-expanded", "true");
  await page.keyboard.press("Escape");
  await expect(input).toHaveAttribute("aria-expanded", "false");
  await expect(listbox).toBeHidden();
});

test("time controls expose accessible names", async ({ page }) => {
  await openTimeWorkbench(page);
  const input = page.getByRole("combobox", { name: "Add IANA time zone" });
  const listbox = page.getByRole("listbox", { name: "Time zone suggestions" });

  await expect(input).toHaveAttribute("aria-controls", "time-zone-suggestions");
  await expect(page.getByRole("button", { name: "Add time zone" })).toBeVisible();
  await input.fill("America/New_York");
  await expect(listbox).toBeVisible();
  await expect(page.getByRole("option", { name: "America/New_York" })).toBeVisible();
  await page.getByRole("button", { name: "Add time zone" }).click();
  await expect(page.getByRole("button", { name: "Remove America/New_York" })).toBeVisible();
});

test("time result copies structured JSON with accessible feedback", async ({ page, context }) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await openTimeWorkbench(page);

  await page.locator("#time-instant-input").fill("0");
  await page.locator("#time-convert-submit").click();
  await expect(page.locator("#time-conversion-output tbody tr")).toHaveCount(1);
  await page.getByRole("button", { name: "Copy time result" }).click();

  await expect(page.getByRole("button", { name: "Copied" })).toBeVisible();
  await expect(page.locator("#status")).toHaveText("Copied time result");
  const clipboard = await page.evaluate(() => navigator.clipboard.readText());
  const copiedResult = JSON.parse(clipboard);
  expect(copiedResult.instant.utc_rfc3339).toBe("1970-01-01T00:00:00Z");
  expect(copiedResult.zones[0].zone).toBe("UTC");
});

test("time workbench supports keyboard tab navigation and form submission", async ({ page }) => {
  await openWorkbench(page);

  await page.locator("#module-identifiers").focus();
  await page.keyboard.press("ArrowRight");
  await expect(page.locator("#module-time")).toBeFocused();
  await expect(page.locator("#module-time")).toHaveAttribute("aria-selected", "true");
  await page.locator("#time-mode-convert").focus();
  await page.keyboard.press("ArrowRight");
  await expect(page.locator("#time-mode-resolve")).toBeFocused();
  await expect(page.locator("#time-mode-resolve")).toHaveAttribute("aria-selected", "true");
  await page.locator("#time-local-input").fill("2026-11-01T01:30:00");
  await page.locator("#time-resolve-zone").fill("America/New_York");
  await page.locator("#time-local-input").press("Enter");

  await expect(page.locator("#time-resolution-status")).toHaveText("Fold");
});
