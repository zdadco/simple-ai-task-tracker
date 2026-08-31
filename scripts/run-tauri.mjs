import { spawn } from 'child_process';
import { homedir } from 'os';
import { join } from 'path';

const cargoBin = join(homedir(), '.cargo', 'bin');
const sep = process.platform === 'win32' ? ';' : ':';
if (!process.env.PATH.split(sep).some((p) => p.toLowerCase() === cargoBin.toLowerCase())) {
  process.env.PATH = `${process.env.PATH}${sep}${cargoBin}`;
}

const args = process.argv.slice(2);
const child = spawn('npx', ['tauri', ...args], {
  stdio: 'inherit',
  shell: true,
  env: process.env,
});

child.on('error', (err) => {
  console.error(err);
  process.exit(1);
});

child.on('exit', (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code ?? 1);
});
