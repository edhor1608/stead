import { describe, test, expect, beforeEach, afterEach } from 'bun:test';
import { mkdir, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { verifyCommand } from './verify.ts';
import { writeContract, readContract } from '../storage/yaml.ts';
import type { Contract } from '../schema/contract.ts';

describe('verifyCommand', () => {
  let testDir: string;

  beforeEach(async () => {
    testDir = join(tmpdir(), `stead-test-${Date.now()}-${Math.random().toString(36).slice(2)}`);
    await mkdir(testDir, { recursive: true });
  });

  afterEach(async () => {
    await rm(testDir, { recursive: true, force: true });
  });

  const createTestContract = (overrides: Partial<Contract> = {}): Contract => ({
    id: `test-${Date.now().toString(36)}`,
    task: 'test task',
    verification: 'echo ok',
    status: 'pending',
    created_at: new Date().toISOString(),
    completed_at: null,
    output: null,
    ...overrides,
  });

  test('loads contract and runs verification', async () => {
    const contract = createTestContract({ verification: 'echo hello' });
    await writeContract(contract, testDir);

    const result = await verifyCommand(contract.id, testDir);

    expect(result.contract.id).toBe(contract.id);
    expect(result.passed).toBe(true);
  });

  test('updates status to passed on exit 0', async () => {
    const contract = createTestContract({ verification: 'true' });
    await writeContract(contract, testDir);

    const result = await verifyCommand(contract.id, testDir);

    expect(result.passed).toBe(true);
    expect(result.contract.status).toBe('passed');
    expect(result.contract.completed_at).not.toBeNull();

    // Verify persisted to storage
    const stored = await readContract(contract.id, testDir);
    expect(stored?.status).toBe('passed');
  });

  test('updates status to failed on non-zero exit', async () => {
    const contract = createTestContract({ verification: 'false' });
    await writeContract(contract, testDir);

    const result = await verifyCommand(contract.id, testDir);

    expect(result.passed).toBe(false);
    expect(result.contract.status).toBe('failed');
    expect(result.contract.completed_at).not.toBeNull();

    // Verify persisted to storage
    const stored = await readContract(contract.id, testDir);
    expect(stored?.status).toBe('failed');
  });

  test('throws error for missing contract', async () => {
    await expect(verifyCommand('non-existent', testDir)).rejects.toThrow(
      'Contract not found: non-existent'
    );
  });

  test('captures verification output', async () => {
    const contract = createTestContract({ verification: 'echo "test output"' });
    await writeContract(contract, testDir);

    const result = await verifyCommand(contract.id, testDir);

    expect(result.contract.output).toContain('test output');
  });

  test('captures stderr in output', async () => {
    const contract = createTestContract({ verification: 'echo "error message" >&2' });
    await writeContract(contract, testDir);

    const result = await verifyCommand(contract.id, testDir);

    expect(result.contract.output).toContain('error message');
  });

  test('captures both stdout and stderr', async () => {
    const contract = createTestContract({
      verification: 'echo "stdout"; echo "stderr" >&2',
    });
    await writeContract(contract, testDir);

    const result = await verifyCommand(contract.id, testDir);

    expect(result.contract.output).toContain('stdout');
    expect(result.contract.output).toContain('stderr');
  });
});
