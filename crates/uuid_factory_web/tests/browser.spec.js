import { expect, test } from "@playwright/test";

async function openWorkbench(page) {
  await page.goto("/");
  await expect(page.locator("body")).toHaveAttribute("data-ready", "true");
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
  await page.locator("#generate-submit").click();

  expect(externalRequests).toEqual([]);
  expect(consoleErrors).toEqual([]);
  expect(pageErrors).toEqual([]);
});

test("keyboard focus is visible and status updates are announced", async ({ page }) => {
  await openWorkbench(page);

  await expect(page.locator("#status")).toHaveAttribute("aria-live", "polite");
  await page.keyboard.press("Tab");
  await expect(page.locator("#mode-generate")).toBeFocused();
  const focusStyle = await page.locator("#mode-generate").evaluate((element) => {
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
