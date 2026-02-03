import { describe, test, expect, beforeEach, afterEach } from 'bun:test';
import { mkdir, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { showCommand } from './show.ts';
import { writeContract } from '../storage/yaml.ts';
import type { Contract } from '../schema/contract.ts';

describe('showCommand', () => {
  let testDir: string;

  beforeEach(async () => {
    testDir = join(tmpdir(), `stead-test-${Date.now()}-${Math.random().toString(36).slice(2)}`);
    await mkdir(testDir, { recursive: true });
  });

  afterEach(async () => {
    await rm(testDir, { recursive: true, force: true });
  });

  const createTestContract = (overrides: Partial<Contract> = {}): Contract => ({
    id: 'abc123-xyz',
    task: 'fix the login bug',
    verification: 'bun test auth',
    status: 'passed',
    created_at: '2026-02-02T14:30:00.000Z',
    completed_at: '2026-02-02T14:35:22.000Z',
    output: '✓ auth.test.ts\n  ✓ login succeeds with valid credentials\n  ✓ login fails with invalid password',
    ...overrides,
  });

  test('shows contract details', async () => {
    const contract = createTestContract();
    await writeContract(contract, testDir);

    const output = await showCommand(contract.id, testDir);

    expect(output).toContain('Contract: abc123-xyz');
    expect(output).toContain('Status: passed');
    expect(output).toContain('Task: fix the login bug');
    expect(output).toContain('Verification: bun test auth');
    expect(output).toContain('Created: 2026-02-02 14:30:00');
    expect(output).toContain('Completed: 2026-02-02 14:35:22');
    expect(output).toContain('Output:');
    expect(output).toContain('✓ auth.test.ts');
  });

  test('returns "Contract not found" for missing ID', async () => {
    await mkdir(join(testDir, '.stead', 'contracts'), { recursive: true });

    const output = await showCommand('nonexistent-id', testDir);

    expect(output).toBe('Contract not found: nonexistent-id');
  });

  test('formats timestamps nicely', async () => {
    const contract = createTestContract({
      created_at: '2026-12-25T08:05:03.000Z',
      completed_at: '2026-12-25T09:15:45.000Z',
    });
    await writeContract(contract, testDir);

    const output = await showCommand(contract.id, testDir);

    expect(output).toContain('Created: 2026-12-25 08:05:03');
    expect(output).toContain('Completed: 2026-12-25 09:15:45');
  });

  test('handles pending contract without completed_at or output', async () => {
    const contract = createTestContract({
      status: 'pending',
      completed_at: null,
      output: null,
    });
    await writeContract(contract, testDir);

    const output = await showCommand(contract.id, testDir);

    expect(output).toContain('Status: pending');
    expect(output).not.toContain('Completed:');
    expect(output).not.toContain('Output:');
  });
});
