import { listContracts } from '../storage/yaml.ts';
import type { Contract, ContractStatus } from '../schema/contract.ts';

const VALID_STATUSES: ContractStatus[] = ['pending', 'running', 'passed', 'failed'];

/**
 * List contracts with optional status filtering.
 * Returns formatted table output.
 */
export async function listCommand(statusFilter?: string, cwd?: string): Promise<string> {
  if (statusFilter && !VALID_STATUSES.includes(statusFilter as ContractStatus)) {
    throw new Error(`Invalid status '${statusFilter}'. Valid values: ${VALID_STATUSES.join(', ')}`);
  }

  let contracts = await listContracts(cwd);

  if (statusFilter) {
    contracts = contracts.filter((c) => c.status === statusFilter);
  }

  if (contracts.length === 0) {
    return 'No contracts found';
  }

  return formatTable(contracts);
}

function formatTable(contracts: Contract[]): string {
  const COL_ID = 15;
  const COL_STATUS = 9;
  const COL_TASK = 30;

  const header = [
    pad('ID', COL_ID),
    pad('STATUS', COL_STATUS),
    pad('TASK', COL_TASK),
    'CREATED',
  ].join('');

  const rows = contracts.map((c) => {
    const created = formatDate(c.created_at);
    return [
      pad(c.id, COL_ID),
      pad(c.status, COL_STATUS),
      pad(truncate(c.task, COL_TASK - 2), COL_TASK),
      created,
    ].join('');
  });

  return [header, ...rows].join('\n');
}

function pad(str: string, width: number): string {
  return str.padEnd(width);
}

function truncate(str: string, maxLen: number): string {
  if (str.length <= maxLen) return str;
  return str.slice(0, maxLen - 3) + '...';
}

function formatDate(isoDate: string): string {
  const d = new Date(isoDate);
  const year = d.getUTCFullYear();
  const month = String(d.getUTCMonth() + 1).padStart(2, '0');
  const day = String(d.getUTCDate()).padStart(2, '0');
  const hour = String(d.getUTCHours()).padStart(2, '0');
  const min = String(d.getUTCMinutes()).padStart(2, '0');
  return `${year}-${month}-${day} ${hour}:${min}`;
}
