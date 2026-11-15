#!/usr/bin/env node
// Builds the SvelteKit frontend for a Tauri bundle.
//
// The main `tauri.conf.json` finds the frontend with a POSIX shell loop
// (`sh -c 'while [ ! -f ... ]'`), which assumes a shell that Windows does not
// ship. This script walks the same path in Node, which is always present
// because the frontend itself is a Node project, then runs the frontend build.
//
// Tauri runs `beforeBuildCommand` from the directory that holds the config, so
// the walk starts there and climbs to the repository root.
import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, join, parse } from "node:path";
import { fileURLToPath } from "node:url";

/** Returns the `frontend` directory that owns a package.json, walking upward. */
function findFrontend(start) {
  const { root } = parse(start);
  let dir = start;
  for (;;) {
    const candidate = join(dir, "frontend", "package.json");
    if (existsSync(candidate)) return join(dir, "frontend");
    if (dir === root) return null;
    dir = dirname(dir);
  }
}

// `process.cwd()` is the crate directory Tauri runs the command from.
const frontend = findFrontend(process.cwd());
if (frontend === null) {
  console.error("build-frontend: no frontend/package.json found above", process.cwd());
  process.exit(1);
}

// `shell: true` is what makes this work on Windows, where npm is `npm.cmd`.
const result = spawnSync("npm", ["run", "build"], {
  cwd: frontend,
  stdio: "inherit",
  shell: true,
});

if (result.error) {
  console.error("build-frontend: could not run npm:", result.error.message);
  process.exit(1);
}
process.exit(result.status ?? 1);
