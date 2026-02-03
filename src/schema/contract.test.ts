import { describe, test, expect } from 'bun:test';
import type { Contract, ContractStatus } from './contract.ts';

describe('Contract schema', () => {
  test('can create a pending contract', () => {
    const contract: Contract = {
      id: 'abc123',
      task: 'fix the login bug',
      verification: 'bun test auth',
      status: 'pending',
      created_at: '2024-01-15T10:30:00Z',
      completed_at: null,
      output: null,
    };

    expect(contract.id).toBe('abc123');
    expect(contract.status).toBe('pending');
    expect(contract.completed_at).toBeNull();
  });

  test('can create a running contract', () => {
    const contract: Contract = {
      id: 'def456',
      task: 'add new API endpoint',
      verification: 'curl -f http://localhost:3000/api/new',
      status: 'running',
      created_at: '2024-01-15T11:00:00Z',
      completed_at: null,
      output: null,
    };

    expect(contract.status).toBe('running');
  });

  test('can create a passed contract', () => {
    const contract: Contract = {
      id: 'ghi789',
      task: 'fix TypeScript errors',
      verification: 'bun run typecheck',
      status: 'passed',
      created_at: '2024-01-15T09:00:00Z',
      completed_at: '2024-01-15T09:15:00Z',
      output: 'All checks passed.\n',
    };

    expect(contract.status).toBe('passed');
    expect(contract.completed_at).not.toBeNull();
    expect(contract.output).toContain('passed');
  });

  test('can create a failed contract', () => {
    const contract: Contract = {
      id: 'jkl012',
      task: 'make tests pass',
      verification: 'bun test',
      status: 'failed',
      created_at: '2024-01-15T14:00:00Z',
      completed_at: '2024-01-15T14:30:00Z',
      output: 'Error: 2 tests failed\n  - auth.test.ts:15\n  - user.test.ts:42\n',
    };

    expect(contract.status).toBe('failed');
    expect(contract.output).toContain('Error');
  });

  test('ContractStatus type accepts all valid values', () => {
    const statuses: ContractStatus[] = ['pending', 'running', 'passed', 'failed'];
    expect(statuses).toHaveLength(4);
  });
});
