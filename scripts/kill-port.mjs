#!/usr/bin/env node
/**
 * Kill process(es) listening on a TCP port (dev helper for stuck Vite on 1420).
 * Usage: node scripts/kill-port.mjs [port]
 */
const port = Number(process.argv[2] ?? process.env.VITE_PORT ?? 1420);
if (!Number.isInteger(port) || port < 1 || port > 65535) {
  console.error(`Invalid port: ${process.argv[2]}`);
  process.exit(1);
}

import { execSync } from 'child_process';

function killWindows(p) {
  let out;
  try {
    out = execSync(`netstat -ano | findstr :${p}`, { encoding: 'utf8', stdio: ['pipe', 'pipe', 'ignore'] });
  } catch {
    return false;
  }
  const pids = new Set();
  for (const line of out.split(/\r?\n/)) {
    if (!line.includes('LISTENING')) continue;
    const parts = line.trim().split(/\s+/);
    const pid = Number(parts[parts.length - 1]);
    if (pid > 0) pids.add(pid);
  }
  let killed = false;
  for (const pid of pids) {
    try {
      execSync(`taskkill /PID ${pid} /F`, { stdio: 'ignore' });
      console.log(`Freed port ${p}: killed PID ${pid}`);
      killed = true;
    } catch {
      console.warn(`Could not kill PID ${pid} on port ${p}`);
    }
  }
  return killed;
}

function killUnix(p) {
  let out;
  try {
    out = execSync(`lsof -ti tcp:${p} -sTCP:LISTEN`, { encoding: 'utf8', stdio: ['pipe', 'pipe', 'ignore'] });
  } catch {
    return false;
  }
  const pids = out
    .split(/\s+/)
    .map((s) => Number(s.trim()))
    .filter((n) => n > 0);
  let killed = false;
  for (const pid of pids) {
    try {
      execSync(`kill -9 ${pid}`, { stdio: 'ignore' });
      console.log(`Freed port ${p}: killed PID ${pid}`);
      killed = true;
    } catch {
      console.warn(`Could not kill PID ${pid} on port ${p}`);
    }
  }
  return killed;
}

const killed = process.platform === 'win32' ? killWindows(port) : killUnix(port);
if (!killed) {
  console.log(`Port ${port} is not in use (no listener found).`);
}