import { describe, test, expect, beforeEach, afterEach } from 'bun:test';
import { $ } from 'bun';
import { mkdir, rm, readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

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
    const proc = Bun.spawn(['bun', 'run', './src/cli/index.ts', 'invalid'], {
      stdout: 'pipe',
      stderr: 'pipe',
    });
    const exitCode = await proc.exited;
    const stderr = await new Response(proc.stderr).text();

    expect(exitCode).toBe(1);
    expect(stderr).toContain('Error: Unknown command: invalid');
    expect(stderr).toContain('Run "stead --help" for usage');
  });

  test('missing required args shows error', async () => {
    const proc = Bun.spawn(['bun', 'run', './src/cli/index.ts', 'run'], {
      stdout: 'pipe',
      stderr: 'pipe',
    });
    const exitCode = await proc.exited;
    const stderr = await new Response(proc.stderr).text();

    expect(exitCode).toBe(1);
    expect(stderr).toContain('Error: run command requires a task argument');
  });
});

describe('stead CLI E2E', () => {
  let testDir: string;
  const cliPath = join(process.cwd(), 'src/cli/index.ts');

  beforeEach(async () => {
    testDir = join(tmpdir(), `stead-e2e-${Date.now()}-${Math.random().toString(36).slice(2)}`);
    await mkdir(testDir, { recursive: true });
  });

  afterEach(async () => {
    await rm(testDir, { recursive: true, force: true });
  });

  test('list with no contracts', async () => {
    const result = await $`bun run ${cliPath} list`.cwd(testDir).text();
    expect(result.trim()).toBe('No contracts found');
  });

  test('show with missing contract', async () => {
    const result = await $`bun run ${cliPath} show nonexistent`.cwd(testDir).text();
    expect(result.trim()).toBe('Contract not found: nonexistent');
  });
});
