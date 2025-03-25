import fs from "node:fs";
import path from "node:path";

const mirrors = [
  { host: "gh-proxy.com", prefix: true },
  { host: "kkgithub.com" },
];

const GITHUB = "https://github.com/";

const mirrorContent = (mirror, text) => {
  if (mirror.prefix) {
    return text.replaceAll(
      GITHUB,
      `https://${mirror.host}/https://github.com/`
    );
  }

  return text.replaceAll(GITHUB, `https://${mirror.host}/`);
};

const newMirrorJSON = (text, mirror, filepath) => {
  const content = mirrorContent(mirror, text);
  fs.writeFileSync(filepath, content);
};

const run = async () => {
  let text = process.env.TEXT;

  // Remove leading and trailing quotes
  if (text[0] === '"') {
    text = text.slice(1);
  }

  if (text[text.length - 1] === '"') {
    text = text.slice(0, text.length - 1);
  }

  text = text
    .replace("\\n}", "}") // Handle trailing newline
    .replaceAll("\\n ", "\n") // Remove newlines outside notes
    .replaceAll(/\s{2,}/g, "") // Remove all whitespace
    .replace(/(\\\\+)(?=")/g, "\\") // Replace escaped double quotes
    .replace(/(\\+)(n)/g, "\\n"); // Handle newlines within notes

  const currentDir = process.cwd();
  const targetDir = path.join(currentDir, "mirrors");

  if (!fs.existsSync(targetDir)) {
    fs.mkdirSync(targetDir);
  }

  mirrors.forEach((m, i) =>
    newMirrorJSON(text, m, path.join(targetDir, `latest-mirror-${i + 1}.json`))
  );
};

run();
