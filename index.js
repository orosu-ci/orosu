const core = require("@actions/core");
const exec = require("@actions/exec");
const { promises: fs } = require("fs");
const path = require("path");
const os = require("os");

export async function run() {
  const address = core.getInput("address", { required: true });
  const script = core.getInput("script", { required: true });
  const key = core.getInput("key", { required: true });
  const args = core.getInput("args") || "";

  const platform = os.platform(); // linux, darwin, win32
  const arch = os.arch() === "arm64" ? "arm64" : "amd64";

  const artifact = `orosu-client-${platform}-${arch}${
    platform === "win32" ? ".exe" : ""
  }`;

  core.info(`Platform: ${platform}-${arch}`);

  const binaryPath = path.join("bin", artifact);

  if (platform !== "win32") {
    await fs.chmod(binaryPath, 0o755);
  }

  core.info("Running orosu-client...");

  const cmdArgs = ["--address", address, "--script", script, "--key", key];

  if (args) {
    cmdArgs.push(...args.split(" ").filter((arg) => arg.length > 0));
  }

  await exec.exec(binaryPath, cmdArgs);
}
