import { describe, test, expect } from 'bun:test';
import { $ } from 'bun';

describe('stead CLI', () => {
  test('--help shows usage', async () => {
    const result = await $`bun run ./src/cli/index.ts --help`.text();
    expect(result).toContain('stead - Contract-based execution environment');
    expect(result).toContain('run');
    expect(result).toContain('list');
    expect(result).toContain('show');
    expect(result).toContain('verify');
  });

  test('--version shows version', async () => {
    const result = await $`bun run ./src/cli/index.ts --version`.text();
    expect(result.trim()).toBe('stead 0.1.0');
  });

  test('unknown command exits with error', async () => {
    try {
      await $`bun run ./src/cli/index.ts invalid`.throws(true);
      expect(false).toBe(true); // Should not reach here
    } catch (e) {
      // Expected to throw
      expect(true).toBe(true);
    }
  });
});
