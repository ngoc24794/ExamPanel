import path from 'path'
import { fileURLToPath } from 'url'

const here = path.dirname(fileURLToPath(import.meta.url))

/**
 * Where the specs write screenshots / PDFs. By default under `ui/test-results/` (git-ignored) so
 * running the suite never modifies tracked files (RA-005). Set EXAMPANEL_UPDATE_DOCS=1 to refresh
 * the committed copies under `docs/screenshots` and `docs/reports` on purpose.
 */
export function artifactDir(kind: 'screenshots' | 'reports', phase: string): string {
  return process.env.EXAMPANEL_UPDATE_DOCS === '1'
    ? path.resolve(here, '../../docs', kind, phase)
    : path.resolve(here, '../test-results/artifacts', kind, phase)
}
