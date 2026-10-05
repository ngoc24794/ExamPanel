// Small UI helpers on top of WebdriverIO (all selectors by data-testid or visible text).
import { sleep } from './harness.mjs';
export const tid = (b, id) => b.$(`[data-testid="${id}"]`);
export async function clickTid(b, id, { timeout = 8000 } = {}) { const e = await tid(b, id); await e.waitForExist({ timeout }); await e.waitForClickable({ timeout }).catch(() => {}); await e.click(); }
export async function clickText(b, text, { tag = '*', timeout = 8000 } = {}) {
  const e = await b.$(`//${tag}[normalize-space(.)="${text}" or normalize-space(text())="${text}"]`); await e.waitForExist({ timeout }); await e.click(); }
export async function bodyText(b) { return b.execute(() => document.body.innerText); }
export async function waitText(b, re, timeout = 10000) { const t0 = Date.now(); while (Date.now() - t0 < timeout) { const t = await bodyText(b); if (re.test(t)) return t; await sleep(250); } return null; }
export async function setInput(b, selector, value) {
  const e = await b.$(selector); await e.waitForExist({ timeout: 8000 });
  await b.execute((el, v) => { const proto = el instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype; Object.getOwnPropertyDescriptor(proto, 'value').set.call(el, v); el.dispatchEvent(new Event('input', { bubbles: true })); el.dispatchEvent(new Event('change', { bubbles: true })); }, e, value);
}
export async function toasts(b) { return b.execute(() => [...document.querySelectorAll('[role="status"],[data-sonner-toast],[class*="toast"]')].map((e) => e.innerText)); }

/** Radix select: click the trigger (by current text or index inside container) then pick option text. */
export async function radixSelect(b, triggerEl, optionText) {
  await triggerEl.click(); await sleep(350);
  const opt = await b.$(`//*[@role="option"][normalize-space(.)="${optionText}" or contains(normalize-space(.),"${optionText}")]`);
  await opt.waitForExist({ timeout: 4000 }); await opt.click(); await sleep(250);
}
export const dialogEl = (b) => b.$('[role="dialog"]');
export async function dialogComboboxes(b) { const d = await dialogEl(b); return d.$$('[role="combobox"]'); }
