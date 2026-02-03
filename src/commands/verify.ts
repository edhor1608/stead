/**
 * Verify command: Re-runs a contract's verification command and updates status.
 */

import { readContract, writeContract } from '../storage/yaml.ts';
import type { Contract } from '../schema/contract.ts';

export interface VerifyResult {
  contract: Contract;
  passed: boolean;
}

/** Get cross-platform shell configuration */
function getShellConfig(): { shell: string; flag: string } {
  const isWindows = process.platform === 'win32';
  return {
    shell: isWindows ? 'cmd' : 'sh',
    flag: isWindows ? '/c' : '-c',
  };
}

/**
 * Load a contract by ID, run its verification command, and update status.
 * @param id - Contract ID to verify
 * @param cwd - Working directory (defaults to process.cwd())
 * @returns The updated contract and whether verification passed
 * @throws Error if contract not found
 */
export async function verifyCommand(id: string, cwd = process.cwd()): Promise<VerifyResult> {
  const contract = await readContract(id, cwd);
  if (!contract) {
    throw new Error(`Contract not found: ${id}`);
  }

  // Run verification command
  const { shell, flag } = getShellConfig();
  const proc = Bun.spawn([shell, flag, contract.verification], {
    cwd,
    stdout: 'pipe',
    stderr: 'pipe',
  });

  const [stdout, stderr] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
  ]);

  const exitCode = await proc.exited;
  const passed = exitCode === 0;

  // Update contract
  const updatedContract: Contract = {
    ...contract,
    status: passed ? 'passed' : 'failed',
    completed_at: new Date().toISOString(),
    output: [stdout, stderr].filter(Boolean).join('\n').trim() || null,
  };

  await writeContract(updatedContract, cwd);

  return { contract: updatedContract, passed };
}
