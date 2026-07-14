import { copyFile, mkdir, stat } from "node:fs/promises";
import { dirname, resolve } from "node:path";

const source = resolve("crates/bombo_core/target/wasm32-unknown-unknown/release/bombo_core.wasm");
const target = resolve("public/wasm/bombo_core.wasm");

await stat(source);
await mkdir(dirname(target), { recursive: true });
await copyFile(source, target);
console.log(`copied ${source} -> ${target}`);
