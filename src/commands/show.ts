/**
 * Show command - displays detailed contract information.
 */

import { readContract } from '../storage/yaml.ts';
import type { Contract } from '../schema/contract.ts';

/** Format ISO timestamp to "YYYY-MM-DD HH:mm:ss" */
function formatTimestamp(isoString: string): string {
  const date = new Date(isoString);
  const year = date.getUTCFullYear();
  const month = String(date.getUTCMonth() + 1).padStart(2, '0');
  const day = String(date.getUTCDate()).padStart(2, '0');
  const hours = String(date.getUTCHours()).padStart(2, '0');
  const minutes = String(date.getUTCMinutes()).padStart(2, '0');
  const seconds = String(date.getUTCSeconds()).padStart(2, '0');
  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
}

/** Format a contract for display */
function formatContract(contract: Contract): string {
  const lines: string[] = [
    `Contract: ${contract.id}`,
    `Status: ${contract.status}`,
    `Task: ${contract.task}`,
    `Verification: ${contract.verification}`,
    `Created: ${formatTimestamp(contract.created_at)}`,
  ];

  if (contract.completed_at) {
    lines.push(`Completed: ${formatTimestamp(contract.completed_at)}`);
  }

  if (contract.output) {
    lines.push('');
    lines.push('Output:');
    lines.push(contract.output);
  }

  return lines.join('\n');
}

/** Show details of a contract by ID */
export async function showCommand(id: string, cwd = process.cwd()): Promise<string> {
  const contract = await readContract(id, cwd);

  if (!contract) {
    return `Contract not found: ${id}`;
  }

  return formatContract(contract);
}
