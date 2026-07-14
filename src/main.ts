import "./styles.css";
import { WorldWorkbench } from "./workbench/WorldWorkbench";

const root = document.querySelector<HTMLElement>("#app");
if (!root) throw new Error("Missing #app root");

const workbench = new WorldWorkbench(root);
workbench.start().catch((error) => {
  const fatal = document.createElement("main");
  fatal.className = "fatal";
  const heading = document.createElement("h1");
  heading.textContent = "WorldTools requires WebGPU";
  const details = document.createElement("pre");
  details.textContent = error instanceof Error ? error.message : String(error);
  fatal.append(heading, details);
  root.replaceChildren(fatal);
});

window.addEventListener("beforeunload", () => workbench.dispose(), { once: true });
