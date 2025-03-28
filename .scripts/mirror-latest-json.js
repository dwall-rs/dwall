import fs from "node:fs";
import path from "node:path";

const latestJsonContent = fs.readFileSync(path.join("..", "latest.json"), {
  encoding: "utf8",
});

const mirrors = [
  { host: "gh-proxy.com", prefix: true },
  { host: "kkgithub.com" },
];

const GITHUB = "https://github.com/";

const mirrorContent = (mirror) => {
  if (mirror.prefix) {
    return latestJsonContent.replaceAll(
      GITHUB,
      `https://${mirror.host}/https://github.com/`
    );
  }

  return latestJsonContent.replaceAll(GITHUB, `https://${mirror.host}/`);
};

const newMirrorJSON = (mirror, filepath) => {
  const content = mirrorContent(mirror);
  fs.writeFileSync(filepath, content);
};

const run = async () => {
  const currentDir = process.cwd();
  const targetDir = path.join(currentDir, "mirrors");

  if (!fs.existsSync(targetDir)) {
    fs.mkdirSync(targetDir);
  }

  mirrors.forEach((m, i) =>
    newMirrorJSON(m, path.join(targetDir, `latest-mirror-${i + 1}.json`))
  );
};

run();
