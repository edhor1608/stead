export type ParseResult =
  | { command: 'run'; task: string; verify: string }
  | { command: 'list'; status?: string }
  | { command: 'show'; id: string }
  | { command: 'verify'; id: string }
  | { command: 'help' }
  | { command: 'version' }
  | { error: string };

export function parse(args: string[]): ParseResult {
  if (args.length === 0 || args[0] === '--help' || args[0] === '-h') {
    return { command: 'help' };
  }

  if (args[0] === '--version' || args[0] === '-v') {
    return { command: 'version' };
  }

  const command = args[0];

  switch (command) {
    case 'run':
      return parseRun(args.slice(1));
    case 'list':
      return parseList(args.slice(1));
    case 'show':
      return parseShow(args.slice(1));
    case 'verify':
      return parseVerify(args.slice(1));
    default:
      return { error: `Unknown command: ${command}` };
  }
}

function parseRun(args: string[]): ParseResult {
  let task: string | undefined;
  let verify: string | undefined;

  for (let i = 0; i < args.length; i++) {
    const arg = args[i]!;

    if (arg === '--verify') {
      verify = args[i + 1];
      i += 1;
    } else if (arg.startsWith('--verify=')) {
      verify = arg.slice('--verify='.length);
    } else if (!arg.startsWith('-')) {
      task = arg;
    }
  }

  if (!task) {
    return { error: 'run command requires a task argument' };
  }

  if (!verify) {
    return { error: 'run command requires --verify flag' };
  }

  return { command: 'run', task, verify };
}

function parseList(args: string[]): ParseResult {
  let status: string | undefined;

  for (let i = 0; i < args.length; i++) {
    const arg = args[i]!;

    if (arg === '--status') {
      status = args[i + 1];
      i += 1;
    } else if (arg.startsWith('--status=')) {
      status = arg.slice('--status='.length);
    }
  }

  if (status !== undefined) {
    return { command: 'list', status };
  }

  return { command: 'list' };
}

function parseShow(args: string[]): ParseResult {
  const id = args[0];

  if (!id || id.startsWith('-')) {
    return { error: 'show command requires a contract ID' };
  }

  return { command: 'show', id };
}

function parseVerify(args: string[]): ParseResult {
  const id = args[0];

  if (!id || id.startsWith('-')) {
    return { error: 'verify command requires a contract ID' };
  }

  return { command: 'verify', id };
}
