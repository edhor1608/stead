#!/usr/bin/env bun

// stead - Contract-based execution environment for agent-driven development
// Entry point for CLI

import { parse } from './parser.ts';
import { runCommand } from '../commands/run.ts';
import { listCommand } from '../commands/list.ts';
import { showCommand } from '../commands/show.ts';
import { verifyCommand } from '../commands/verify.ts';

const HELP = `
stead - Contract-based execution environment for agent-driven development

Usage:
  stead run "<task>" --verify "<cmd>"   Create and execute a contract
  stead list [--status=<status>]        List contracts
  stead show <id>                       Show contract details
  stead verify <id>                     Re-run verification

Options:
  --help, -h    Show this help message
  --version     Show version
`;

const VERSION = 'stead 0.1.0';

async function main() {
  const args = process.argv.slice(2);
  const result = parse(args);

  if ('error' in result) {
    console.error(`Error: ${result.error}`);
    console.error('Run "stead --help" for usage');
    process.exit(1);
  }

  switch (result.command) {
    case 'help':
      console.log(HELP);
      break;

    case 'version':
      console.log(VERSION);
      break;

    case 'run': {
      try {
        const contract = await runCommand(result.task, result.verify);
        console.log(`Contract ${contract.id} completed with status: ${contract.status}`);
        if (contract.output) {
          console.log('\nVerification output:');
          console.log(contract.output);
        }
      } catch (err) {
        console.error(`Error running contract: ${err instanceof Error ? err.message : err}`);
        process.exit(1);
      }
      break;
    }

    case 'list': {
      const output = await listCommand(result.status);
      console.log(output);
      break;
    }

    case 'show': {
      const output = await showCommand(result.id);
      console.log(output);
      break;
    }

    case 'verify': {
      try {
        const { contract, passed } = await verifyCommand(result.id);
        console.log(`Contract ${contract.id}: ${passed ? 'PASSED' : 'FAILED'}`);
        if (contract.output) {
          console.log('\nVerification output:');
          console.log(contract.output);
        }
      } catch (err) {
        console.error(`Error: ${err instanceof Error ? err.message : err}`);
        process.exit(1);
      }
      break;
    }
  }
}

main();
