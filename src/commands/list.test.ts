import { describe, test, expect, beforeEach, afterEach } from 'bun:test';
import { mkdir, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { listCommand } from './list.ts';
import { writeContract } from '../storage/yaml.ts';
import type { Contract } from '../schema/contract.ts';

describe('listCommand', () => {
  let testDir: string;

  beforeEach(async () => {
    testDir = join(tmpdir(), `stead-test-${Date.now()}-${Math.random().toString(36).slice(2)}`);
    await mkdir(testDir, { recursive: true });
  });

  afterEach(async () => {
    await rm(testDir, { recursive: true, force: true });
  });

  const createContract = (overrides: Partial<Contract>): Contract => ({
    id: 'abc123-xyz',
    task: 'test task',
    verification: 'echo ok',
    status: 'pending',
    created_at: '2026-02-02T14:30:00Z',
    completed_at: null,
    output: null,
    ...overrides,
  });

  test('returns "No contracts found" when empty', async () => {
    const result = await listCommand(undefined, testDir);
    expect(result).toBe('No contracts found');
  });

  test('lists all contracts', async () => {
    const c1 = createContract({
      id: 'abc123-xyz',
      task: 'fix the login bug',
      status: 'passed',
      created_at: '2026-02-02T14:30:00Z',
    });
    const c2 = createContract({
      id: 'def456-uvw',
      task: 'add unit tests',
      status: 'running',
      created_at: '2026-02-02T15:45:00Z',
    });

    await writeContract(c1, testDir);
    await writeContract(c2, testDir);

    const result = await listCommand(undefined, testDir);

    expect(result).toContain('abc123-xyz');
    expect(result).toContain('def456-uvw');
    expect(result).toContain('fix the login bug');
    expect(result).toContain('add unit tests');
    expect(result).toContain('passed');
    expect(result).toContain('running');
  });

  test('filters by status', async () => {
    const c1 = createContract({
      id: 'abc123-xyz',
      task: 'fix the login bug',
      status: 'passed',
    });
    const c2 = createContract({
      id: 'def456-uvw',
      task: 'add unit tests',
      status: 'running',
    });

    await writeContract(c1, testDir);
    await writeContract(c2, testDir);

    const result = await listCommand('passed', testDir);

    expect(result).toContain('abc123-xyz');
    expect(result).toContain('passed');
    expect(result).not.toContain('def456-uvw');
    expect(result).not.toContain('running');
  });

  test('returns "No contracts found" when filter matches nothing', async () => {
    const c1 = createContract({
      id: 'abc123-xyz',
      status: 'passed',
    });

    await writeContract(c1, testDir);

    const result = await listCommand('failed', testDir);
    expect(result).toBe('No contracts found');
  });

  test('formats output as table with header', async () => {
    const c1 = createContract({
      id: 'abc123-xyz',
      task: 'fix the login bug',
      status: 'passed',
      created_at: '2026-02-02T14:30:00Z',
    });

    await writeContract(c1, testDir);

    const result = await listCommand(undefined, testDir);
    const lines = result.split('\n');

    // Header line
    expect(lines[0]).toContain('ID');
    expect(lines[0]).toContain('STATUS');
    expect(lines[0]).toContain('TASK');
    expect(lines[0]).toContain('CREATED');

    // Data line
    expect(lines[1]).toContain('abc123-xyz');
    expect(lines[1]).toContain('passed');
    expect(lines[1]).toContain('fix the login bug');
    expect(lines[1]).toContain('2026-02-02 14:30');
  });

  test('truncates long task descriptions', async () => {
    const longTask = 'this is a very long task description that should be truncated to fit in the table';
    const c1 = createContract({
      id: 'abc123-xyz',
      task: longTask,
      status: 'pending',
    });

    await writeContract(c1, testDir);

    const result = await listCommand(undefined, testDir);
    // Task column is 30 chars based on spec, should truncate
    expect(result).not.toContain(longTask);
    expect(result.length).toBeLessThan(longTask.length + 100);
  });

  test('throws error for invalid status filter', async () => {
    await expect(listCommand('invalid', testDir)).rejects.toThrow(
      "Invalid status 'invalid'. Valid values: pending, running, passed, failed"
    );
  });
});
