// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";

import { detectInitialLocale, i18n, normalizeLocale, setLocale } from ".";

const LOCALE_STORAGE_KEY = "paperclip.locale";

afterEach(() => {
  window.localStorage.removeItem(LOCALE_STORAGE_KEY);
});

describe("normalizeLocale", () => {
  it("normalizes zh variants to zh-CN / zh-TW", () => {
    expect(normalizeLocale("zh-CN")).toBe("zh-CN");
    expect(normalizeLocale("zh_CN")).toBe("zh-CN");
    expect(normalizeLocale("zh-Hans")).toBe("zh-CN");
    expect(normalizeLocale("zh")).toBe("zh-CN");
    expect(normalizeLocale("zh-TW")).toBe("zh-TW");
    expect(normalizeLocale("zh-Hant")).toBe("zh-TW");
  });

  it("matches supported locales case-insensitively", () => {
    expect(normalizeLocale("fr")).toBe("fr");
    expect(normalizeLocale("PT-BR")).toBe("pt-BR");
    expect(normalizeLocale("ja-JP")).toBeNull();
    expect(normalizeLocale("")).toBeNull();
    expect(normalizeLocale(undefined)).toBeNull();
  });
});

describe("detectInitialLocale", () => {
  it("prefers the stored locale over the browser language", () => {
    window.localStorage.setItem(LOCALE_STORAGE_KEY, "zh-CN");
    expect(detectInitialLocale()).toBe("zh-CN");
  });

  it("falls back to the default locale without a stored choice", () => {
    expect(detectInitialLocale()).toBe("en");
  });
});

describe("setLocale", () => {
  it("persists the normalized locale and switches i18next", async () => {
    setLocale("zh_CN");
    expect(window.localStorage.getItem(LOCALE_STORAGE_KEY)).toBe("zh-CN");
    await i18n.changeLanguage("zh-CN");
    expect(i18n.language.startsWith("zh")).toBe(true);
  });

  it("resolves invalid values to the default locale", () => {
    setLocale("klingon");
    expect(window.localStorage.getItem(LOCALE_STORAGE_KEY)).toBe("en");
  });
});
