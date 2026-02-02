import { describe, test, expect } from 'bun:test';
import { parse, type ParseResult } from './parser';

describe('CLI parser', () => {
  describe('run command', () => {
    test('parses run with --verify flag', () => {
      const result = parse(['run', 'add login feature', '--verify', 'bun test']);
      expect(result).toEqual({
        command: 'run',
        task: 'add login feature',
        verify: 'bun test',
      });
    });

    test('parses run with --verify= syntax', () => {
      const result = parse(['run', 'fix bug', '--verify=npm test']);
      expect(result).toEqual({
        command: 'run',
        task: 'fix bug',
        verify: 'npm test',
      });
    });

    test('returns error when --verify is missing', () => {
      const result = parse(['run', 'some task']);
      expect(result).toEqual({
        error: 'run command requires --verify flag',
      });
    });

    test('returns error when task is missing', () => {
      const result = parse(['run', '--verify', 'bun test']);
      expect(result).toEqual({
        error: 'run command requires a task argument',
      });
    });

    test('returns error when run has no arguments', () => {
      const result = parse(['run']);
      expect(result).toEqual({
        error: 'run command requires a task argument',
      });
    });
  });

  describe('list command', () => {
    test('parses list without options', () => {
      const result = parse(['list']);
      expect(result).toEqual({ command: 'list' });
    });

    test('parses list with --status flag', () => {
      const result = parse(['list', '--status', 'pending']);
      expect(result).toEqual({
        command: 'list',
        status: 'pending',
      });
    });

    test('parses list with --status= syntax', () => {
      const result = parse(['list', '--status=completed']);
      expect(result).toEqual({
        command: 'list',
        status: 'completed',
      });
    });
  });

  describe('show command', () => {
    test('parses show with contract ID', () => {
      const result = parse(['show', 'abc123']);
      expect(result).toEqual({
        command: 'show',
        id: 'abc123',
      });
    });

    test('returns error when ID is missing', () => {
      const result = parse(['show']);
      expect(result).toEqual({
        error: 'show command requires a contract ID',
      });
    });
  });

  describe('verify command', () => {
    test('parses verify with contract ID', () => {
      const result = parse(['verify', 'xyz789']);
      expect(result).toEqual({
        command: 'verify',
        id: 'xyz789',
      });
    });

    test('returns error when ID is missing', () => {
      const result = parse(['verify']);
      expect(result).toEqual({
        error: 'verify command requires a contract ID',
      });
    });
  });

  describe('help command', () => {
    test('parses --help', () => {
      const result = parse(['--help']);
      expect(result).toEqual({ command: 'help' });
    });

    test('parses -h', () => {
      const result = parse(['-h']);
      expect(result).toEqual({ command: 'help' });
    });

    test('parses empty args as help', () => {
      const result = parse([]);
      expect(result).toEqual({ command: 'help' });
    });
  });

  describe('version command', () => {
    test('parses --version', () => {
      const result = parse(['--version']);
      expect(result).toEqual({ command: 'version' });
    });

    test('parses -v', () => {
      const result = parse(['-v']);
      expect(result).toEqual({ command: 'version' });
    });
  });

  describe('unknown command', () => {
    test('returns error for unknown command', () => {
      const result = parse(['unknown']);
      expect(result).toEqual({
        error: "Unknown command: unknown",
      });
    });
  });
});
