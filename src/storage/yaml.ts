/**
 * YAML-based contract storage layer.
 *
 * Contracts are stored as individual YAML files in .stead/contracts/
 * This makes them human-readable and git-trackable.
 */

import { mkdir, readFile, writeFile, readdir, stat } from 'node:fs/promises';
import { join } from 'node:path';
import { stringify, parse } from 'yaml';
import type { Contract } from '../schema/contract.ts';

const STEAD_DIR = '.stead';
const CONTRACTS_DIR = 'contracts';

/** Get the contracts directory path */
export function getContractsDir(cwd = process.cwd()): string {
  return join(cwd, STEAD_DIR, CONTRACTS_DIR);
}

/** Ensure the contracts directory exists */
export async function ensureContractsDir(cwd = process.cwd()): Promise<string> {
  const dir = getContractsDir(cwd);
  await mkdir(dir, { recursive: true });
  return dir;
}

/** Generate a unique contract ID */
export function generateId(): string {
  const timestamp = Date.now().toString(36);
  const random = Math.random().toString(36).substring(2, 8);
  return `${timestamp}-${random}`;
}

/** Get the file path for a contract */
export function getContractPath(id: string, cwd = process.cwd()): string {
  return join(getContractsDir(cwd), `${id}.yaml`);
}

/** Write a contract to storage */
export async function writeContract(contract: Contract, cwd = process.cwd()): Promise<void> {
  await ensureContractsDir(cwd);
  const path = getContractPath(contract.id, cwd);
  const yaml = stringify(contract);
  await writeFile(path, yaml, 'utf-8');
}

/** Read a contract from storage */
export async function readContract(id: string, cwd = process.cwd()): Promise<Contract | null> {
  const path = getContractPath(id, cwd);
  try {
    const content = await readFile(path, 'utf-8');
    return parse(content) as Contract;
  } catch (err) {
    if ((err as NodeJS.ErrnoException).code === 'ENOENT') {
      return null;
    }
    throw err;
  }
}

/** List all contracts in storage */
export async function listContracts(cwd = process.cwd()): Promise<Contract[]> {
  const dir = getContractsDir(cwd);

  try {
    const files = await readdir(dir);
    const yamlFiles = files.filter((f) => f.endsWith('.yaml'));

    const contracts: Contract[] = [];
    for (const file of yamlFiles) {
      const id = file.replace('.yaml', '');
      const contract = await readContract(id, cwd);
      if (contract) {
        contracts.push(contract);
      }
    }

    // Sort by created_at descending (newest first)
    return contracts.sort(
      (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
    );
  } catch (err) {
    if ((err as NodeJS.ErrnoException).code === 'ENOENT') {
      return [];
    }
    throw err;
  }
}

/** Check if contracts directory exists */
export async function contractsDirExists(cwd = process.cwd()): Promise<boolean> {
  try {
    const dir = getContractsDir(cwd);
    const stats = await stat(dir);
    return stats.isDirectory();
  } catch {
    return false;
  }
}
