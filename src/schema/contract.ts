/**
 * Contract schema types for stead CLI.
 *
 * A contract wraps an agent task with verification, providing
 * structured tracking of what was attempted and whether it worked.
 */

/** Status of a contract through its lifecycle */
export type ContractStatus = 'pending' | 'running' | 'passed' | 'failed';

/** A contract defining an agent task with verification */
export interface Contract {
  /** Unique identifier for the contract */
  id: string;

  /** Human-readable task description passed to the agent */
  task: string;

  /** Command that exits 0 on success, non-zero on failure */
  verification: string;

  /** Current status of the contract */
  status: ContractStatus;

  /** ISO 8601 timestamp when contract was created */
  created_at: string;

  /** ISO 8601 timestamp when contract completed (passed or failed), null if not yet complete */
  completed_at: string | null;

  /** Captured stdout/stderr from the verification command, null if not yet run */
  output: string | null;
}
