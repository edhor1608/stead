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

/** Custom error for storage-related failures */
export class StorageError extends Error {
  public readonly path?: string;

  constructor(message: string, cause?: unknown, path?: string) {
    super(message, { cause });
    this.name = 'StorageError';
    this.path = path;
  }
}

const STEAD_DIR = '.stead';
const CONTRACTS_DIR = 'contracts';

/** Get the contracts directory path */
export function getContractsDir(cwd = process.cwd()): string {
  return join(cwd, STEAD_DIR, CONTRACTS_DIR);
}

/** Ensure the contracts directory exists */
export async function ensureContractsDir(cwd = process.cwd()): Promise<string> {
  const dir = getContractsDir(cwd);
  try {
    await mkdir(dir, { recursive: true });
    return dir;
  } catch (err) {
    const code = (err as NodeJS.ErrnoException).code;
    if (code === 'EACCES' || code === 'EPERM') {
      throw new StorageError(
        `Permission denied: cannot create contracts directory at ${dir}`,
        err,
        dir
      );
    }
    throw new StorageError(
      `Failed to create contracts directory at ${dir}: ${(err as Error).message}`,
      err,
      dir
    );
  }
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

  let yaml: string;
  try {
    yaml = stringify(contract);
  } catch (err) {
    throw new StorageError(
      `Failed to serialize contract ${contract.id}: ${(err as Error).message}`,
      err,
      path
    );
  }

  try {
    await writeFile(path, yaml, 'utf-8');
  } catch (err) {
    const code = (err as NodeJS.ErrnoException).code;
    if (code === 'EACCES' || code === 'EPERM') {
      throw new StorageError(
        `Permission denied: cannot write contract to ${path}`,
        err,
        path
      );
    }
    if (code === 'ENOSPC') {
      throw new StorageError(`Disk full: cannot write contract to ${path}`, err, path);
    }
    throw new StorageError(
      `Failed to write contract to ${path}: ${(err as Error).message}`,
      err,
      path
    );
  }
}

/** Read a contract from storage */
export async function readContract(id: string, cwd = process.cwd()): Promise<Contract | null> {
  const path = getContractPath(id, cwd);

  let content: string;
  try {
    content = await readFile(path, 'utf-8');
  } catch (err) {
    const code = (err as NodeJS.ErrnoException).code;
    if (code === 'ENOENT') {
      return null;
    }
    if (code === 'EACCES' || code === 'EPERM') {
      throw new StorageError(`Permission denied: cannot read contract at ${path}`, err, path);
    }
    throw new StorageError(
      `Failed to read contract at ${path}: ${(err as Error).message}`,
      err,
      path
    );
  }

  try {
    return parse(content) as Contract;
  } catch (err) {
    throw new StorageError(
      `Corrupted contract file at ${path}: invalid YAML syntax`,
      err,
      path
    );
  }
}

/**
 * List all contracts in storage.
 *
 * Gracefully skips corrupted contract files, logging warnings but continuing
 * to load valid contracts. This prevents one bad file from breaking the entire list.
 */
export async function listContracts(cwd = process.cwd()): Promise<Contract[]> {
  const dir = getContractsDir(cwd);

  let files: string[];
  try {
    files = await readdir(dir);
  } catch (err) {
    const code = (err as NodeJS.ErrnoException).code;
    if (code === 'ENOENT') {
      return [];
    }
    if (code === 'EACCES' || code === 'EPERM') {
      throw new StorageError(
        `Permission denied: cannot read contracts directory at ${dir}`,
        err,
        dir
      );
    }
    throw new StorageError(
      `Failed to read contracts directory at ${dir}: ${(err as Error).message}`,
      err,
      dir
    );
  }

  const yamlFiles = files.filter((f) => f.endsWith('.yaml'));
  const contracts: Contract[] = [];
  const errors: { id: string; error: string }[] = [];

  for (const file of yamlFiles) {
    const id = file.replace('.yaml', '');
    try {
      const contract = await readContract(id, cwd);
      if (contract) {
        contracts.push(contract);
      }
    } catch (err) {
      // Skip corrupted files but track them for potential reporting
      errors.push({
        id,
        error: err instanceof StorageError ? err.message : (err as Error).message,
      });
    }
  }

  // Log warnings for corrupted files (could be expanded to return these)
  if (errors.length > 0) {
    console.warn(
      `Warning: ${errors.length} contract file(s) could not be loaded:`,
      errors.map((e) => `\n  - ${e.id}: ${e.error}`).join('')
    );
  }

  // Sort by created_at descending (newest first)
  return contracts.sort(
    (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
  );
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
