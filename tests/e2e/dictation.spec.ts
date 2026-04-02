import { test, expect } from "@playwright/test";

test.describe("Handy01 Dictation", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
  });

  test("should display app title", async ({ page }) => {
    await expect(page.getByText("Handy01")).toBeVisible();
  });

  test("should show idle state initially", async ({ page }) => {
    await expect(page.getByText("Idle")).toBeVisible();
  });

  test("should show settings button", async ({ page }) => {
    await expect(page.getByText("Settings")).toBeVisible();
  });

  test("should open settings panel", async ({ page }) => {
    await page.getByText("Settings").click();
    await expect(page.getByText("Correction Style")).toBeVisible();
    await expect(page.getByText("Whisper Model")).toBeVisible();
  });

  test("should show correction style options", async ({ page }) => {
    await expect(page.getByText("inline")).toBeVisible();
    await expect(page.getByText("highlighted")).toBeVisible();
    await expect(page.getByText("draft-final")).toBeVisible();
  });

  test("should switch correction styles", async ({ page }) => {
    await page.getByText("highlighted").click();
    const preview = page.locator(".highlighted-correction");
    await expect(preview).toBeVisible();
  });

  test("should show placeholder text when idle", async ({ page }) => {
    await expect(
      page.getByText("Press Ctrl+Shift+Space to start"),
    ).toBeVisible();
  });

  test("should display status indicators", async ({ page }) => {
    await expect(page.getByText("Sidecar")).toBeVisible();
  });
});
