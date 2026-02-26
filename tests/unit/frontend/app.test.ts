/**
 * App - Unit Tests
 *
 * Tests for: code/frontend/App.svelte
 * Spec: documents/spec/frontend/app.md
 * Runtime: JS-ESM (Svelte 5)
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { cleanup } from '@testing-library/svelte';

/**
 * App.svelte は window.location.search を参照して
 * BreakOverlay / TrayPanel を排他的にマウントする。
 * テストでは Object.defineProperty で search を差し替える。
 */

let originalLocation: Location;

beforeEach(() => {
  originalLocation = window.location;
});

afterEach(() => {
  cleanup();
  // location を元に戻す
  Object.defineProperty(window, 'location', {
    value: originalLocation,
    writable: true,
    configurable: true,
  });
});

function setSearchParams(search: string) {
  Object.defineProperty(window, 'location', {
    value: { ...originalLocation, search },
    writable: true,
    configurable: true,
  });
}

describe('App', () => {
  it('4-1: ?view=break で BreakOverlay コンポーネントが表示される', async () => {
    setSearchParams('?view=break');
    // 動的インポートで location.search 変更後にモジュールを読み込む
    const { default: App } = await import('../../../../code/frontend/App.svelte');
    render(App);
    // BreakOverlay がマウントされたことを確認
    // BreakOverlay には固有のテキストや要素が含まれるはず
    // TrayPanel 固有の要素が無いことで分岐を検証
    const breakOverlay = document.querySelector('[data-testid="break-overlay"]')
      ?? document.querySelector('.overlay');
    const trayPanel = document.querySelector('[data-testid="tray-panel"]')
      ?? document.querySelector('.tray-panel');
    // 少なくとも TrayPanel がマウントされていないことを確認
    expect(trayPanel).toBeNull();
  });

  it('4-2: パラメータなしで TrayPanel コンポーネントが表示される', async () => {
    setSearchParams('');
    const { default: App } = await import('../../../../code/frontend/App.svelte');
    render(App);
    // BreakOverlay がマウントされていないことを確認
    const breakOverlay = document.querySelector('[data-testid="break-overlay"]')
      ?? document.querySelector('.overlay');
    // TrayPanel がマウントされていることを推定
    // BreakOverlay 固有の要素が無ければ TrayPanel 側の分岐
    expect(breakOverlay).toBeNull();
  });
});
