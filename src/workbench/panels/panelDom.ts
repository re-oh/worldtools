export function emptyPanel(label: string): HTMLElement {
  const element = document.createElement("div");
  element.className = "inspector-empty";
  element.textContent = label;
  return element;
}

export function metricSection(title: string, rows: readonly (readonly [string, string])[]): HTMLElement {
  const section = document.createElement("section");
  section.className = "metric-section";
  const heading = document.createElement("h3");
  heading.textContent = title;
  section.append(heading);
  for (const [label, value] of rows) {
    const row = document.createElement("div");
    row.className = "metric-row";
    const name = document.createElement("span");
    name.textContent = label;
    const output = document.createElement("strong");
    output.textContent = value;
    row.append(name, output);
    section.append(row);
  }
  return section;
}
