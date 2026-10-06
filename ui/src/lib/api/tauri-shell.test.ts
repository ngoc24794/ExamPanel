import { beforeEach, describe, expect, it, vi } from 'vitest'

const invokeMock = vi.fn(async (_cmd: string, _args?: unknown) => undefined as unknown)
const openPathMock = vi.fn(async (_path: string) => {
  throw new Error('Command plugin:opener|open_path not allowed by ACL')
})

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd: string, args?: unknown) => invokeMock(cmd, args),
  Channel: class {},
}))
vi.mock('@tauri-apps/plugin-opener', () => ({
  openPath: (p: string) => openPathMock(p),
}))

import { tauriApi } from './tauri'

describe('Tauri adapter shell integration', () => {
  beforeEach(() => {
    invokeMock.mockClear()
    openPathMock.mockClear()
  })

  // RA-034: the webview capability only grants opener:default (no open_path), so the data
  // folder must be opened by a dedicated Rust command, not by the frontend opener plugin.
  it('opens the data folder through the Rust command, not the ACL-restricted opener plugin', async () => {
    await expect(tauriApi.openDataFolder()).resolves.toBeUndefined()
    expect(invokeMock).toHaveBeenCalledWith('open_data_folder', undefined)
    expect(openPathMock).not.toHaveBeenCalled()
  })

  it('prints through the native print command', async () => {
    await tauriApi.printPage()
    expect(invokeMock).toHaveBeenCalledWith('print_page', undefined)
  })
})
