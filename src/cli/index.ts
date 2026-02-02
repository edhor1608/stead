#!/usr/bin/env bun

// stead - Contract-based execution environment for agent-driven development
// Entry point for CLI

const args = process.argv.slice(2);

if (args.length === 0 || args[0] === '--help' || args[0] === '-h') {
  console.log(`
stead - Contract-based execution environment for agent-driven development

Usage:
  stead run "<task>" --verify "<cmd>"   Create and execute a contract
  stead list [--status=<status>]        List contracts
  stead show <id>                       Show contract details
  stead verify <id>                     Re-run verification

Options:
  --help, -h    Show this help message
  --version     Show version
`);
  process.exit(0);
}

if (args[0] === '--version') {
  console.log('stead 0.1.0');
  process.exit(0);
}

const command = args[0];

switch (command) {
  case 'run':
  case 'list':
  case 'show':
  case 'verify':
    console.log(`Command '${command}' not yet implemented`);
    process.exit(1);
  default:
    console.error(`Unknown command: ${command}`);
    console.error('Run "stead --help" for usage');
    process.exit(1);
}
