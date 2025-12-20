const core = require("@actions/core");
const exec = require("@actions/exec");
const { promises: fs } = require("fs");
const path = require("path");
const os = require("os");

async function run() {
  try {
    const address = core.getInput("address", { required: true });
    const script = core.getInput("script", { required: true });
    const key = core.getInput("key", { required: true });
    const args = core.getInput("args") || "";

    const platform = os.platform(); // linux, darwin, win32
    const arch = os.arch() === "arm64" ? "arm64" : "amd64";

    const artifact = `orosu-client-${platform}-${arch}${
      platform === "win32" ? ".exe" : ""
    }`;

    const actionDir = path.resolve(__dirname, "..");
    const binaryPath = path.join(actionDir, "bin", artifact);
    
    core.info(`Binary path: ${binaryPath}`);

    if (platform !== "win32") {
      await fs.chmod(binaryPath, 0o755);
    }

    core.info("Running orosu-client...");
    console.error(`Executing: ${binaryPath} --address ${address} --script ${script} --key [REDACTED] ${args}`);
    console.log(`Executing: ${binaryPath} --address ${address} --script ${script} --key [REDACTED] ${args}`);

    const cmdArgs = ["--address", address, "--script", script, "--key", key];

    if (args) {
      cmdArgs.push(...args.split(" ").filter((arg) => arg.length > 0));
    }

    const exitCode = await exec.exec(binaryPath, cmdArgs, {
      ignoreReturnCode: true,
    });
    
    if (exitCode !== 0) {
      process.exit(exitCode);
    }
  } catch (error) {
    core.setFailed(error.message);
    process.exit(1);
  }
}

module.exports = { run };
