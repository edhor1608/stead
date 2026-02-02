/**
 * The 'run' command - creates and executes contracts.
 *
 * Flow:
 * 1. Create contract from task and verification command
 * 2. Write to storage (status: pending)
 * 3. Update to running status
 * 4. Execute claude CLI with the task
 * 5. Run verification command
 * 6. Update status to passed/failed based on exit code
 * 7. Capture verification output
 */

import type { Contract, ContractStatus } from '../schema/contract.ts';
import { writeContract, generateId } from '../storage/yaml.ts';

/** Options for spawning the claude CLI - allows mocking in tests */
export interface SpawnClaudeResult {
  exitCode: number;
  output: string;
}

export interface RunOptions {
  /** Custom claude spawner for testing */
  spawnClaude?: (task: string) => Promise<SpawnClaudeResult>;
  /** Callback when status changes (for testing lifecycle) */
  onStatusChange?: (status: ContractStatus) => void;
}

/** Default implementation that calls the real claude CLI */
async function defaultSpawnClaude(task: string): Promise<SpawnClaudeResult> {
  const proc = Bun.spawn(['claude', '-p', task], {
    stdout: 'pipe',
    stderr: 'pipe',
  });

  const [stdout, stderr] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
  ]);

  const exitCode = await proc.exited;
  return { exitCode, output: stdout + stderr };
}

/** Run a verification command and capture output */
async function runVerification(cmd: string): Promise<{ exitCode: number; output: string }> {
  const proc = Bun.spawn(['sh', '-c', cmd], {
    stdout: 'pipe',
    stderr: 'pipe',
  });

  const [stdout, stderr] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
  ]);

  const exitCode = await proc.exited;
  return { exitCode, output: stdout + stderr };
}

/** Create a new contract */
function createContract(task: string, verification: string): Contract {
  return {
    id: generateId(),
    task,
    verification,
    status: 'pending',
    created_at: new Date().toISOString(),
    completed_at: null,
    output: null,
  };
}

/**
 * Execute the run command.
 *
 * Creates a contract, executes the claude CLI with the task,
 * runs verification, and updates contract status.
 */
export async function runCommand(
  task: string,
  verifyCmd: string,
  options: RunOptions = {}
): Promise<Contract> {
  const spawnClaude = options.spawnClaude ?? defaultSpawnClaude;
  const onStatusChange = options.onStatusChange ?? (() => {});

  // 1. Create contract
  const contract = createContract(task, verifyCmd);

  // 2. Write to storage (status: pending)
  onStatusChange('pending');
  await writeContract(contract);

  // 3. Update to running status
  contract.status = 'running';
  onStatusChange('running');
  await writeContract(contract);

  // 4. Execute claude CLI
  await spawnClaude(task);

  // 5. Run verification command
  const verification = await runVerification(verifyCmd);

  // 6. Update status based on verification exit code
  contract.status = verification.exitCode === 0 ? 'passed' : 'failed';
  contract.completed_at = new Date().toISOString();

  // 7. Capture verification output
  contract.output = verification.output;

  // Final write
  await writeContract(contract);

  return contract;
}
