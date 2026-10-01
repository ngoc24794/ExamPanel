import { describe, it, expect, beforeEach } from 'vitest'
import { MockExamPanelApi } from './mock'

describe('MockExamPanelApi', () => {
  let api: MockExamPanelApi

  beforeEach(() => {
    localStorage.clear()
    api = new MockExamPanelApi()
  })

  it('ping returns expected mock string', async () => {
    const res = await api.ping()
    expect(res).toContain('pong from Mock Engine')
  })

  it('getAppInfo returns app metadata with mock mode', async () => {
    const info = await api.getAppInfo()
    expect(info.name).toBe('ExamPanel')
    expect(info.identifier).toBe('vn.exampanel.app')
    expect(info.mode).toBe('mock')
  })

  it('persists and retrieves theme', async () => {
    expect(await api.getTheme()).toBeNull()
    await api.setTheme('dark')
    expect(await api.getTheme()).toBe('dark')
    await api.setTheme('light')
    expect(await api.getTheme()).toBe('light')
  })
})
