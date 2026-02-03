import { describe, test, expect, beforeEach, afterEach, mock, spyOn } from 'bun:test';
import { mkdir, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { runCommand } from './run.ts';
import { readContract } from '../storage/yaml.ts';

describe('runCommand', () => {
  let testDir: string;
  let originalCwd: () => string;

  beforeEach(async () => {
    // Create a unique temp directory for each test
    testDir = join(tmpdir(), `stead-test-${Date.now()}-${Math.random().toString(36).slice(2)}`);
    await mkdir(testDir, { recursive: true });

    // Mock process.cwd to return our test directory
    originalCwd = process.cwd;
    process.cwd = () => testDir;
  });

  afterEach(async () => {
    process.cwd = originalCwd;
    await rm(testDir, { recursive: true, force: true });
  });

  test('creates a contract with correct fields', async () => {
    const contract = await runCommand('add tests', 'true', {
      spawnClaude: async () => ({ exitCode: 0, output: '' }),
    });

    expect(contract.task).toBe('add tests');
    expect(contract.verification).toBe('true');
    expect(contract.id).toMatch(/^[a-z0-9]+-[a-z0-9]+$/);
    expect(contract.created_at).toMatch(/^\d{4}-\d{2}-\d{2}T/);
  });

  test('updates status through lifecycle', async () => {
    const statuses: string[] = [];

    const contract = await runCommand('task', 'true', {
      spawnClaude: async () => ({ exitCode: 0, output: '' }),
      onStatusChange: (status) => statuses.push(status),
    });

    // Should have gone through: pending -> running -> (then passed as final)
    expect(statuses).toContain('pending');
    expect(statuses).toContain('running');
    expect(contract.status).toBe('passed');
  });

  test('handles verification pass (exit 0)', async () => {
    const contract = await runCommand('task', 'true', {
      spawnClaude: async () => ({ exitCode: 0, output: '' }),
    });

    expect(contract.status).toBe('passed');
    expect(contract.completed_at).not.toBeNull();
  });

  test('handles verification fail (non-zero exit)', async () => {
    const contract = await runCommand('task', 'false', {
      spawnClaude: async () => ({ exitCode: 0, output: '' }),
    });

    expect(contract.status).toBe('failed');
    expect(contract.completed_at).not.toBeNull();
  });

  test('captures verification output', async () => {
    const contract = await runCommand('task', 'echo "hello world"', {
      spawnClaude: async () => ({ exitCode: 0, output: '' }),
    });

    expect(contract.output).toContain('hello world');
  });

  test('persists contract to storage', async () => {
    const contract = await runCommand('persisted task', 'true', {
      spawnClaude: async () => ({ exitCode: 0, output: '' }),
    });

    const stored = await readContract(contract.id, testDir);
    expect(stored).not.toBeNull();
    expect(stored?.task).toBe('persisted task');
    expect(stored?.status).toBe('passed');
  });

  test('calls claude CLI with the task', async () => {
    let capturedTask = '';

    await runCommand('my specific task', 'true', {
      spawnClaude: async (task) => {
        capturedTask = task;
        return { exitCode: 0, output: '' };
      },
    });

    expect(capturedTask).toBe('my specific task');
  });
});
