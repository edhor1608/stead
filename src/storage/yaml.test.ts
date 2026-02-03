import { describe, test, expect, beforeEach, afterEach } from 'bun:test';
import { mkdir, rm, readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import {
  writeContract,
  readContract,
  listContracts,
  generateId,
  getContractsDir,
  ensureContractsDir,
  contractsDirExists,
} from './yaml.ts';
import type { Contract } from '../schema/contract.ts';

describe('YAML Storage', () => {
  let testDir: string;

  beforeEach(async () => {
    // Create a unique temp directory for each test
    testDir = join(tmpdir(), `stead-test-${Date.now()}-${Math.random().toString(36).slice(2)}`);
    await mkdir(testDir, { recursive: true });
  });

  afterEach(async () => {
    // Clean up
    await rm(testDir, { recursive: true, force: true });
  });

  const createTestContract = (overrides: Partial<Contract> = {}): Contract => ({
    id: generateId(),
    task: 'test task',
    verification: 'echo ok',
    status: 'pending',
    created_at: new Date().toISOString(),
    completed_at: null,
    output: null,
    ...overrides,
  });

  describe('generateId', () => {
    test('generates unique IDs', () => {
      const id1 = generateId();
      const id2 = generateId();
      expect(id1).not.toBe(id2);
    });

    test('ID format is timestamp-random', () => {
      const id = generateId();
      expect(id).toMatch(/^[a-z0-9]+-[a-z0-9]+$/);
    });
  });

  describe('ensureContractsDir', () => {
    test('creates .stead/contracts directory', async () => {
      await ensureContractsDir(testDir);
      const exists = await contractsDirExists(testDir);
      expect(exists).toBe(true);
    });

    test('is idempotent', async () => {
      await ensureContractsDir(testDir);
      await ensureContractsDir(testDir);
      const exists = await contractsDirExists(testDir);
      expect(exists).toBe(true);
    });
  });

  describe('writeContract + readContract', () => {
    test('round-trip: write and read contract', async () => {
      const contract = createTestContract();
      await writeContract(contract, testDir);
      const read = await readContract(contract.id, testDir);
      expect(read).toEqual(contract);
    });

    test('returns null for non-existent contract', async () => {
      await ensureContractsDir(testDir);
      const read = await readContract('non-existent', testDir);
      expect(read).toBeNull();
    });

    test('writes valid YAML file', async () => {
      const contract = createTestContract();
      await writeContract(contract, testDir);

      const path = join(getContractsDir(testDir), `${contract.id}.yaml`);
      const content = await readFile(path, 'utf-8');

      expect(content).toContain(`id: ${contract.id}`);
      expect(content).toContain('task: test task');
      expect(content).toContain('verification: echo ok');
      expect(content).toContain('status: pending');
    });
  });

  describe('listContracts', () => {
    test('returns empty array when no contracts', async () => {
      await ensureContractsDir(testDir);
      const contracts = await listContracts(testDir);
      expect(contracts).toEqual([]);
    });

    test('returns empty array when directory does not exist', async () => {
      const contracts = await listContracts(testDir);
      expect(contracts).toEqual([]);
    });

    test('lists all contracts', async () => {
      const c1 = createTestContract({ created_at: '2026-01-01T00:00:00Z' });
      const c2 = createTestContract({ created_at: '2026-01-02T00:00:00Z' });
      const c3 = createTestContract({ created_at: '2026-01-03T00:00:00Z' });

      await writeContract(c1, testDir);
      await writeContract(c2, testDir);
      await writeContract(c3, testDir);

      const contracts = await listContracts(testDir);
      expect(contracts.length).toBe(3);
    });

    test('returns contracts sorted by created_at descending', async () => {
      const c1 = createTestContract({ created_at: '2026-01-01T00:00:00Z' });
      const c2 = createTestContract({ created_at: '2026-01-03T00:00:00Z' });
      const c3 = createTestContract({ created_at: '2026-01-02T00:00:00Z' });

      await writeContract(c1, testDir);
      await writeContract(c2, testDir);
      await writeContract(c3, testDir);

      const contracts = await listContracts(testDir);
      expect(contracts[0]!.id).toBe(c2.id); // newest
      expect(contracts[1]!.id).toBe(c3.id);
      expect(contracts[2]!.id).toBe(c1.id); // oldest
    });
  });
});
